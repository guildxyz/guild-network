use futures::StreamExt;
use gn_client::runtime::oracle::events::OracleRequest;
use gn_client::{
    query,
    tx::{self, Signer},
};
use gn_client::{Api, GuildCall, SubxtError};
use gn_common::identities::IdentityMap;
use gn_common::utils::{matches_variant, verification_msg};
use gn_common::{RequestData, RequestIdentifier};
use sp_keyring::AccountKeyring;
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
    /// Set operator account
    #[structopt(long = "id", default_value = "alice")]
    id: String,
    /// Register as an oracle operator before starting to listen to events
    #[structopt(long)]
    register: bool,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(opt.log)).init();

    let url = format!("ws://{}:{}", opt.node_ip, opt.node_port);

    // TODO: this will be read from the oracle's wallet for testing purposes we
    // are choosing from pre-funded accounts
    let signer = Arc::new(Signer::new(
        match opt.id.to_lowercase().as_str() {
            "bob" => AccountKeyring::Bob,
            "charlie" => AccountKeyring::Charlie,
            "dave" => AccountKeyring::Dave,
            "eve" => AccountKeyring::Eve,
            "ferdie" => AccountKeyring::Ferdie,
            _ => AccountKeyring::Alice,
        }
        .pair(),
    ));

    let api = Api::from_url(&url)
        .await
        .expect("failed to start api client");

    if opt.register {
        tx::send_tx_in_block(api.clone(), &tx::register_operator(), Arc::clone(&signer))
            .await
            .expect("failed to register operator");

        log::info!("operator registration request submitted");
    }

    let mut subscription = api
        .blocks()
        .subscribe_best()
        .await
        .expect("failed to subscribe to blocks");

    while let Some(maybe_block) = subscription.next().await {
        match maybe_block {
            Ok(block) => match block.events().await {
                Ok(maybe_events) => {
                    maybe_events
                        .iter()
                        .filter_map(|maybe_event| {
                            maybe_event.unwrap().as_event::<OracleRequest>().unwrap()
                        })
                        .for_each(|oracle_request| {
                            submit_answer(api.clone(), Arc::clone(&signer), oracle_request)
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
        RequestData::Register(identities) => {
            log::info!("[registration request] acc: {}", oracle_request.requester);
            // deserialize user identities
            let expected_msg = verification_msg(&oracle_request.requester);
            match identities
                .iter()
                .map(|id| id.verify(&expected_msg))
                .collect::<Result<Vec<_>, _>>()
            {
                Ok(_) => true,
                Err(error) => {
                    log::warn!("identity check failed: {}", error);
                    false
                }
            }
        }
        RequestData::ReqCheck {
            account,
            guild,
            role,
        } => {
            log::info!(
                "[requirement check request] acc: {}, guild: {:?}, role: {:?}",
                account,
                guild,
                role
            );
            // fetch requirements
            let requirements_with_logic = query::requirements(api.clone(), guild, role).await?;
            // build requireemnt tree from logic
            let requirement_tree = requiem::LogicTree::from_str(&requirements_with_logic.logic)
                .map_err(|e| SubxtError::Other(e.to_string()))?;
            let identity_map =
                IdentityMap::from_identities(query::user_identity(api.clone(), &account).await?);
            let requirement_futures = requirements_with_logic
                .requirements
                .iter()
                .map(|req| req.check(&identity_map))
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
