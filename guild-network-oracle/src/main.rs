use codec::Encode;
use futures::StreamExt;
use guild_network_client::queries::join_request;
use guild_network_client::runtime::chainlink::events::OracleRequest;
use guild_network_client::transactions::{oracle_callback, send_tx_ready};
use guild_network_client::{Api, FilteredEvents, GuildCall, Signer};
use guild_network_common::JoinRequestWithAccess;
use log::{error, info, trace};
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
        if let Err(e) = try_main(api.clone(), Arc::clone(&signer), &mut events).await {
            error!("{e}");
        }
    }
}

async fn try_main(
    api: Api,
    signer: Arc<Signer>,
    events: &mut FilteredEvents<'_, (OracleRequest,)>,
) -> Result<(), anyhow::Error> {
    trace!("[+] Listening for events");
    if let Some(event) = events.next().await {
        let OracleRequest {
            request_id,
            operator,
            callback,
            fee,
        } = event?.event;
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

        // TODO spawn does not work for multiple calls
        // when I send 10 requests it only responds to one
        //tokio::spawn(async move {
        // TODO storage query
        // TODO verify user identities
        // NOTE unwrap is fine because data has the correct length and
        // will always fit
        let join_request = join_request(api.clone(), request_id).await?;
        info!(
            "guild: {:?}, role: {:?}",
            join_request.guild_name, join_request.role_name
        );

        // TODO retrieve balances and check requirements
        let requirement_check = true;
        let result = JoinRequestWithAccess {
            access: requirement_check,
            requester: join_request.requester,
            requester_identities: join_request.requester_identities,
            guild_name: join_request.guild_name,
            role_name: join_request.role_name,
        };

        let tx = oracle_callback(request_id, result.encode());
        send_tx_ready(api, &tx, signer).await?;
        info!(
            "oracle answer ({}) submitted: {}",
            request_id, requirement_check
        );
        //   Ok::<(), anyhow::Error>(())
        //});
    }

    Ok(())
}

fn matches_variant<T>(a: &T, b: &T) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}
