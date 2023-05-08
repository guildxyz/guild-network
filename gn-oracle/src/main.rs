#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(unused_crate_dependencies)]

use futures::{future::try_join_all, StreamExt};
use gn_api::{
    query,
    tx::{self, Signer},
    AccountId, Api, Balance, BlockNumber, Events, OracleRequestEvent,
};
use sp_core::crypto::{ExposeSecret, SecretString, Zeroize};
use structopt::StructOpt;

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

#[derive(StructOpt)]
#[structopt(name = "Guild Network CLI")]
pub struct Opt {
    /// Set logging level
    #[structopt(short, long, default_value = "info")]
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
    #[structopt(long = "password", default_value = "")]
    password: SecretString,
    /// Activate operator before starting to listen to events
    #[structopt(long)]
    activate: bool,
}

#[tokio::main]
async fn main() {
    let mut opt = Opt::from_args();

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(opt.log)).init();

    let url = format!("ws://{}:{}", opt.node_ip, opt.node_port);
    let (api, signer) = tx::api_with_signer(url, &opt.seed, Some(opt.password.expose_secret()))
        .await
        .expect("failed to initialize client and signer");

    opt.seed.zeroize();

    log::info!("signer account: {}", signer.account_id());

    if opt.activate {
        activate(api.clone(), Arc::clone(&signer)).await;
    }

    run(api, signer).await
}

pub async fn activate(api: Api, operator: Arc<Signer>) {
    if !query::oracle::is_registered(api.clone(), operator.account_id())
        .await
        .expect("failed to fetch operator info")
    {
        panic!("{} is not registered as an operator", operator.account_id());
    }

    tx::send::in_block(api.clone(), &tx::activate_operator(), Arc::clone(&operator))
        .await
        .expect("failed to activate operator");

    log::info!("operator activation request submitted");
}

pub async fn run(api: Api, operator: Arc<Signer>) {
    let active = query::oracle::active_operators(api.clone())
        .await
        .expect("failed to fetch active operators");

    if !active.contains(operator.account_id()) {
        panic!(
            "{} not activated, start oracle with the '--activate' flag",
            operator.account_id()
        );
    } else {
        log::info!("node activated, listening to events...");
    }

    let mut subscription = api
        .blocks()
        .subscribe_best()
        .await
        .expect("failed to subscribe to blocks");

    while let Some(block_result) = subscription.next().await {
        match block_result {
            Ok(block) => {
                match block.events().await {
                    Ok(events) => {
                        let request_ids = OracleRequestIds::new(events, operator.account_id());
                        // query identity + requirement
                        // Redis
                        //submit_answers(api.clone(), Arc::clone(&operator), requests)
                    }
                    Err(err) => log::error!("invalid block events: {err}"),
                }
            }
            Err(err) => log::error!("invalid block: {err}"),
        }
    }
    log::error!("block subscription aborted");
}

struct RequestIds {
    challenge: Vec<u64>,
    join: Vec<u64>,
}

impl RequestIds {
    fn new(events: Events, operator: &AccountId) -> Self {
        let mut challenge = Vec::new();
        let mut join = Vec::new();
        for event_result in events.iter() {
            let maybe_request = event_result
                .ok()
                .map(|event| event.as_event::<OracleRequestEvent>().ok().flatten())
                .flatten();
            if let Some(request) = maybe_request {
                if operator == &request.operator {
                    log::trace!("request not delegated to us");
                } else {
                    log::trace!("OracleRequest: {}, {}", request.request_id, request.fee);
                    match request.pallet_index {
                        gn_common::PALLET_GUILD_IDENTITY_ID => challenge.push(request.request_id),
                        gn_common::PALLET_GUILD_ID => join.push(request.request_id),
                        _ => log::warn!("invalid pallet index: {}", request.pallet_index),
                    }
                }
            }
        }
        Self { challenge, join }
    }

    fn process(self, api: Api, signer: Arc<Signer>) {
        tokio::spawn(async move {
            match try_join_all(answer_futures).await {
                Ok(answers) => {
                    if let Err(e) = tx::send::batch(api, answers.iter(), signer).await {
                        log::warn!("failed to send oracle answers: {}", e)
                    }
                }
                Err(e) => log::warn!("failed to compile oracle answers: {}", e),
            }
        });
    }
}

/*
async fn compile_answer(
    api: Api,
    request_id: RequestIdentifier,
) -> Result<OracleCallback, SubxtError> {
    let oracle_request = query::oracle_request(api.clone(), request_id).await?;

    let oracle_answer = match oracle_request.data {
        RequestData::Register {
            identity_with_auth,
            index: _,
        } => {
            log::info!("[registration request] acc: {}", oracle_request.requester);
            // deserialize user identities
            let expected_msg = verification_msg(&oracle_request.requester);
            identity_with_auth.verify(expected_msg)
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
    log::info!("oracle answer ({}): {:?}", request_id, result);
    Ok(tx::oracle_callback(request_id, result))
}
*/
