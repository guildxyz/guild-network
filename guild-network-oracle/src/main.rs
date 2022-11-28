use futures::StreamExt;
use guild_network_client::queries::{join_request, requirements};
use guild_network_client::runtime::chainlink::events::OracleRequest;
use guild_network_client::transactions::{oracle_callback, send_tx_ready};
use guild_network_client::{cbor_deserialize, signed_msg};
use guild_network_client::{Api, FilteredEvents, GuildCall, Signer};
use guild_network_gate::identities::{Identity, IdentityAuth};
use log::{error, info, warn};
use sp_keyring::AccountKeyring;
use structopt::StructOpt;

use std::sync::Arc;

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
            Ok(oracle_request) => submit_answer(api.clone(), Arc::clone(&signer), oracle_request),
            Err(e) => error!("{e}"),
        }
    }
}

fn submit_answer(api: Api, signer: Arc<Signer>, request: OracleRequest) {
    tokio::spawn(async move {
        if let Err(e) = try_submit_answer(api, signer, request).await {
            error!("{e}");
        }
    });
}

async fn try_submit_answer(
    api: Api,
    signer: Arc<Signer>,
    request: OracleRequest,
) -> Result<(), anyhow::Error> {
    let OracleRequest {
        request_id,
        operator,
        callback,
        fee,
    } = request;
    info!(
        "OracleRequest: {}, {}, {:?}, {}",
        request_id, operator, callback, fee
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
    info!(
        "guild: {:?}, role: {:?}",
        join_request.guild_name, join_request.role_name
    );

    // verify user identities
    let mut identity_check = true;
    let identities: Vec<Identity> = cbor_deserialize(&join_request.requester_identities)?;
    let auths: Vec<IdentityAuth> = cbor_deserialize(&join_request.request_data)?;
    for (identity, auth) in identities.iter().zip(&auths) {
        let expected_msg = signed_msg(
            &join_request.requester,
            &join_request.guild_name,
            &join_request.role_name,
        );
        if identity.verify(auth, Some(&expected_msg)).is_err() {
            identity_check = false;
            warn!("identity {:?} verification failed", identity);
        }
    }

    // TODO retrieve balances and check requirements
    let requirements =
        requirements(api.clone(), join_request.guild_name, join_request.role_name).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(request_id)).await;
    let requirement_check = true;
    let access = identity_check && requirement_check;
    let result = vec![u8::from(access)];
    let tx = oracle_callback(request_id, result);
    send_tx_ready(api, &tx, signer).await?;
    info!(
        "oracle answer ({}) submitted: {}",
        request_id, requirement_check
    );
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

fn matches_variant<T>(a: &T, b: &T) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}
