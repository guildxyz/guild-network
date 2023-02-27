#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(unused_crate_dependencies)]

use futures::StreamExt;
use gn_client::runtime::oracle::events::OracleRequest;
use gn_client::{
    query,
    tx::{self, Signer},
};
use gn_client::{Api, GuildCall, SubxtError};
use gn_common::identity::Identity;
use gn_common::utils::{matches_variant, verification_msg};
use gn_common::{RequestData, RequestIdentifier};
use structopt::StructOpt;

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

const TX_RETRIES: u64 = 10;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Client params",
    about = "Advanced parameters for the Substrate client."
)]
struct Opt {
    /// Set logging level
    #[structopt(short, long, default_value = "warn")]
    log: String,
    /// Set node IP address
    #[structopt(short = "i", long = "node-ip", default_value = "127.0.0.1")]
    node_ip: String,
    /// Set node port number
    #[structopt(short = "p", long = "node-port", default_value = "9944")]
    node_port: String,
    /// Set operator account seed
    #[structopt(long = "seed", default_value = "//Alice")]
    seed: String,
    /// Set operator account password
    #[structopt(long = "password")]
    password: Option<String>,
    /// Activate operator before starting to listen to events
    #[structopt(long)]
    activate: bool,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(opt.log)).init();

    let url = format!("ws://{}:{}", opt.node_ip, opt.node_port);

    let (api, operator) = tx::api_with_signer(url, &opt.seed, opt.password.as_deref())
        .await
        .expect("failed to initialize api and signer");

    log::info!("Public key: {}", operator.account_id());

    if !query::is_operator_registered(api.clone(), operator.account_id())
        .await
        .expect("failed to fetch operator info")
    {
        panic!("{} is not registered as an operator", operator.account_id());
    }

    if opt.activate {
        tx::send_tx_in_block(api.clone(), &tx::activate_operator(), Arc::clone(&operator))
            .await
            .expect("failed to activate operator");

        log::info!("operator activation request submitted");
    }

    let active = query::active_operators(api.clone())
        .await
        .expect("failed to fetch active operators");
    if !active.contains(operator.account_id()) {
        panic!(
            "{} not activated. Run oracle with the '--activate' flag",
            operator.account_id()
        );
    }

    let mut subscription = api
        .blocks()
        .subscribe_best()
        .await
        .expect("failed to subscribe to blocks");

    while let Some(block_result) = subscription.next().await {
        match block_result {
            Ok(block) => match block.events().await {
                Ok(events) => {
                    events
                        .iter()
                        .filter_map(|event_result| event_result.ok())
                        .filter_map(|event_details| {
                            event_details.as_event::<OracleRequest>().ok().flatten()
                        })
                        .for_each(|oracle_request| {
                            submit_answer(api.clone(), Arc::clone(&operator), oracle_request)
                        });
                }
                Err(err) => log::error!("invalid block events: {err}"),
            },
            Err(err) => log::error!("invalid block: {err}"),
        }
    }
    log::error!("block subscription aborted");
}

fn submit_answer(api: Api, signer: Arc<Signer>, request: OracleRequest) {
    tokio::spawn(async move {
        let OracleRequest {
            request_id,
            operator,
            callback,
            fee,
        } = request;

        log::info!(
            "OracleRequest: {}, {}, {:?}, {}",
            request_id,
            operator,
            callback,
            fee
        );

        if &operator != signer.account_id() {
            // request wasn't delegated to us so return
            log::trace!("request not delegated to us");
            return;
        }

        // check whether the incoming request originates from the guild
        // pallet just for testing basically
        if !matches_variant(&callback, &GuildCall::callback { result: vec![] }) {
            log::trace!("callback mismatch");
            return;
        }

        if let Err(e) = try_submit_answer(api, signer, request_id).await {
            log::error!("{e}");
        }
    });
}

async fn try_submit_answer(
    api: Api,
    signer: Arc<Signer>,
    request_id: RequestIdentifier,
) -> Result<(), SubxtError> {
    let oracle_request = query::oracle_request(api.clone(), request_id).await?;

    let oracle_answer = match oracle_request.data {
        RequestData::Register {
            identity_with_auth,
            index: _,
        } => {
            log::info!("[registration request] acc: {}", oracle_request.requester);
            // deserialize user identities
            let expected_msg = verification_msg(&oracle_request.requester);
            identity_with_auth.verify(&expected_msg)
        }
        RequestData::ReqCheck {
            account,
            guild_name,
            role_name,
        } => {
            log::info!(
                "[requirement check request] acc: {}, guild: {:?}, role: {:?}",
                account,
                guild_name,
                role_name,
            );
            // fetch requirements
            let requirements_with_logic = query::requirements(api.clone(), guild_name, role_name)
                .await?
                .ok_or(SubxtError::Other("no requirements found".to_string()))?;
            // build requireemnt tree from logic
            let requirement_tree = requiem::LogicTree::from_str(&requirements_with_logic.logic)
                .map_err(|e| SubxtError::Other(e.to_string()))?;
            let identities = query::user_identity(api.clone(), &account).await?;
            let maybe_address = identities
                .iter()
                .find(|&x| matches_variant(x, &Identity::Address20([0u8; 20])));

            if let Some(address) = maybe_address {
                let requirement_futures = requirements_with_logic
                    .requirements
                    .iter()
                    .map(|req| req.check(address))
                    .collect::<Vec<_>>();
                match futures::future::try_join_all(requirement_futures).await {
                    Ok(boolean_vec) => {
                        let requirement_check_map: HashMap<u32, bool> = boolean_vec
                            .into_iter()
                            .enumerate()
                            .map(|(i, b)| (i as u32, b))
                            .collect();
                        requirement_tree
                            .evaluate(&requirement_check_map)
                            .unwrap_or(false)
                    }
                    Err(error) => {
                        log::warn!("requirement check failed: {}", error);
                        false
                    }
                }
            } else {
                log::warn!("requirement check failed: no registered evm identity");
                false
            }
        }
    };

    let result = vec![u8::from(oracle_answer)];
    let tx = tx::oracle_callback(request_id, result);
    let mut retries = 1;
    while retries <= TX_RETRIES {
        match tx::send_tx_ready(api.clone(), &tx, Arc::clone(&signer)).await {
            Ok(()) => {
                log::info!(
                    "oracle answer ({}) submitted: {}",
                    request_id,
                    oracle_answer
                );
                break;
            }
            Err(error) => {
                log::warn!("submitting transaction returned error: {}", error);
                tokio::time::sleep(tokio::time::Duration::from_millis(retries)).await;
                retries += 1;
            }
        }
    }
    Ok(())
}
