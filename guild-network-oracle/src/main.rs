use futures::StreamExt;
use guild_network_client::queries::{join_request, requirements};
use guild_network_client::runtime::chainlink::events::OracleRequest;
use guild_network_client::transactions::{oracle_callback, send_tx_ready};
use guild_network_client::{cbor_deserialize, Api, FilteredEvents, GuildCall, Signer};
use guild_network_common::unpad_from_32_bytes;
use guild_network_gate::identities::{IdentityMap, IdentityWithAuth};
use guild_network_gate::verification_msg;
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

    let join_request = join_request(api.clone(), request_id).await?;
    log::info!(
        "guild: {:?}, role: {:?}",
        join_request.guild_name,
        join_request.role_name
    );

    // deserialize user identities
    let identities: Vec<IdentityWithAuth> = cbor_deserialize(&join_request.requester_identities)?;
    let expected_msg = verification_msg(
        &join_request.requester,
        unpad_from_32_bytes(&join_request.guild_name),
        unpad_from_32_bytes(&join_request.role_name),
    );

    // fetch requirements
    let requirements_with_logic =
        requirements(api.clone(), join_request.guild_name, join_request.role_name).await?;

    let requirement_tree = requiem::LogicTree::from_str(&requirements_with_logic.logic)?;

    // check identities and requirements
    let mut identity_check = false;
    let mut requirement_check = false;
    match IdentityMap::from_verified_identities(identities, &expected_msg) {
        Ok(identity_map) => {
            identity_check = true;
            let requirement_futures = requirements_with_logic
                .requirements
                .iter()
                .map(|req| req.check(&client, &identity_map))
                .collect::<Vec<_>>();
            // requirement checks
            match futures::future::try_join_all(requirement_futures).await {
                Ok(boolean_vec) => {
                    let requirement_check_map: HashMap<u32, bool> = boolean_vec
                        .into_iter()
                        .enumerate()
                        .map(|(i, b)| (i as u32, b))
                        .collect();
                    requirement_check = requirement_tree
                        .evaluate(&requirement_check_map)
                        .unwrap_or(false);
                }
                Err(error) => log::warn!("identity check failed: {}", error),
            }
        }
        Err(error) => log::warn!("identity check failed: {}", error),
    }

    let access = identity_check && requirement_check;
    let result = vec![u8::from(access)];
    let tx = oracle_callback(request_id, result);
    let mut retries = 1;
    while retries <= TX_RETRIES {
        match send_tx_ready(api.clone(), &tx, Arc::clone(&signer)).await {
            Ok(()) => {
                log::info!(
                    "oracle answer ({}) submitted: {}",
                    request_id,
                    requirement_check
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
