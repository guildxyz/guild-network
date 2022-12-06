use futures::StreamExt;
use guild_network_client::queries::{oracle_request, requirements, user_identity};
use guild_network_client::runtime::chainlink::events::OracleRequest;
use guild_network_client::transactions::{oracle_callback, send_tx_ready};
use guild_network_client::{Api, FilteredEvents, GuildCall, Signer};
use guild_network_common::identities::IdentityMap;
use guild_network_common::utils::{matches_variant, verification_msg};
use guild_network_common::RequestData;
use reqwest::Client as ReqwestClient;
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
}

#[tokio::main]
async fn main() -> ! {
    let opt = Opt::from_args();

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(opt.log)).init();

    let url = format!("ws://{}:{}", opt.node_ip, opt.node_port);
    let client = ReqwestClient::new();

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

    let api = Api::from_url(url)
        .await
        .expect("failed to start api client");

    let mut events = api
        .events()
        .subscribe()
        .await
        .expect("failed to subscribe to events")
        .filter_events::<(OracleRequest,)>();

    loop {
        match next_event(&mut events).await {
            Ok(oracle_request) => submit_answer(
                api.clone(),
                client.clone(),
                Arc::clone(&signer),
                oracle_request,
            ),
            Err(e) => log::error!("{e}"),
        }
    }
}

fn submit_answer(api: Api, client: ReqwestClient, signer: Arc<Signer>, request: OracleRequest) {
    tokio::spawn(async move {
        if let Err(e) = try_submit_answer(api, client, signer, request).await {
            log::error!("{e}");
        }
    });
}

async fn try_submit_answer(
    api: Api,
    client: ReqwestClient,
    signer: Arc<Signer>,
    request: OracleRequest,
) -> Result<(), anyhow::Error> {
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
        return Ok(());
    }

    // check whether the incoming request originates from the guild
    // pallet just for testing basically
    if !matches_variant(&callback, &GuildCall::callback { result: vec![] }) {
        return Ok(());
    }

    let oracle_request = oracle_request(api.clone(), request_id).await?;

    let oracle_answer = match oracle_request.data {
        RequestData::Register(identities) => {
            log::info!("user registration: {}", oracle_request.requester);
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
        RequestData::Join { guild, role } => {
            log::info!("join guild request: {:?}, role: {:?}", guild, role);
            // fetch requirements
            let requirements_with_logic = requirements(api.clone(), guild, role).await?;
            // build requireemnt tree from logic
            let requirement_tree = requiem::LogicTree::from_str(&requirements_with_logic.logic)?;
            let identity_map = IdentityMap::from_identities(
                user_identity(api.clone(), &oracle_request.requester).await?,
            );
            let requirement_futures = requirements_with_logic
                .requirements
                .iter()
                .map(|req| req.check(&client, &identity_map))
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
                    log::warn!("identity check failed: {}", error);
                    false
                }
            }
        }
    };

    let result = vec![u8::from(oracle_answer)];
    let tx = oracle_callback(request_id, result);
    let mut retries = 1;
    while retries <= TX_RETRIES {
        match send_tx_ready(api.clone(), &tx, Arc::clone(&signer)).await {
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

async fn next_event(
    events: &mut FilteredEvents<'_, (OracleRequest,)>,
) -> Result<OracleRequest, anyhow::Error> {
    if let Some(event) = events.next().await {
        let oracle_request = event?.event;
        Ok(oracle_request)
    } else {
        Err(anyhow::anyhow!("next event is None"))
    }
}
