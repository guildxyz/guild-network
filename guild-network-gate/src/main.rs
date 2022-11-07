fn main() {}
/*
use ethers::types::U256;
use futures::StreamExt;
use log::{error, info, trace};
use sp_keyring::AccountKeyring;
use std::sync::Arc;
use structopt::StructOpt;
use substrate_oracle_client::node::chainlink::events::OracleRequest;
use substrate_oracle_client::requirements::process_request;
use substrate_oracle_client::types::*;
use subxt::{
    config::{SubstrateConfig, WithExtrinsicParams},
    dynamic::Value,
    tx::{BaseExtrinsicParams, PairSigner, PlainTip, Signer},
};

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
    let signer = {
        use AccountKeyring::*;

        match opt.id.to_lowercase().as_str() {
            "bob" => Bob,
            "charlie" => Charlie,
            "dave" => Dave,
            "eve" => Eve,
            "ferdie" => Ferdie,
            _ => Alice,
        }
    }
    .pair();

    let signer = Arc::new(PairSigner::new(signer));

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

async fn try_main<T>(
    api: Api,
    signer: Arc<T>,
    events: &mut FilteredEvents<'_, (OracleRequest,)>,
) -> Result<(), anyhow::Error>
where
    T: Signer<WithExtrinsicParams<SubstrateConfig, BaseExtrinsicParams<SubstrateConfig, PlainTip>>>
        + Send
        + Sync
        + 'static,
{
    trace!("[+] Listening for events");
    if let Some(event) = events.next().await {
        match event?.event {
            OracleRequest(operator_id, request_id, _, _, data, _, _) => {
                info!("OracleRequest: {}, {}, {:?}", operator_id, request_id, data);
                if &operator_id != signer.account_id() {
                    // request wasn't delegated to us so return
                    return Ok(());
                }

                let mut guild_id_bytes = [0u8; 8];
                guild_id_bytes.copy_from_slice(&data[0..8]);
                let guild_id = u64::from_le_bytes(guild_id_bytes);

                tokio::spawn(async move {
                    let storage_query = subxt::dynamic::storage(
                        "Guild",
                        "Guilds",
                        vec![Value::u128(guild_id as u128)],
                    );
                    let minimum_balance = api
                        .storage()
                        .fetch_or_default(&storage_query, None)
                        .await?
                        .as_u128()
                        .unwrap_or_default()
                        .to_string();
                    let tx = process_request(
                        request_id,
                        U256::from_dec_str(&minimum_balance)?,
                        &data[8..],
                    )
                    .await?;
                    let hash = api
                        .tx()
                        .sign_and_submit_default(&tx, signer.as_ref())
                        .await?;
                    info!("oracle answer submitted, hash: {}", hash);
                    Ok::<(), anyhow::Error>(())
                });
            }
        }
    }

    Ok(())
}
*/
