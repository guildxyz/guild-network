use anyhow::anyhow;
use ethereum_types::{Address, Signature as EvmSignature};
use ethers_core::types::Signature as EthSignature;

use futures::StreamExt;
use guild_network_client::runtime::chainlink::events::OracleRequest;
use guild_network_client::transactions::{oracle_callback, send_tx_ready};
use guild_network_client::{Api, FilteredEvents, Signer};
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

// TODO: Substitute address's type after restructuring
pub fn verify(
    substr_addr: &Vec<u8>,
    eth_address: Address,
    signature: EvmSignature,
) -> Result<(), anyhow::Error> {
    let msg = std::str::from_utf8(substr_addr).map_err(|e| anyhow!(e))?;
    let ethers_signature = EthSignature::try_from(signature.as_bytes()).map_err(|e| anyhow!(e))?;
    ethers_signature
        .verify(msg, eth_address.to_fixed_bytes())
        .map_err(|e| anyhow!(e))?;

    Ok(())
}

async fn try_main(
    api: Api,
    signer: Arc<Signer>,
    events: &mut FilteredEvents<'_, (OracleRequest,)>,
) -> Result<(), anyhow::Error> {
    trace!("[+] Listening for events");
    if let Some(event) = events.next().await {
        match event?.event {
            OracleRequest(operator_id, request_id, _, _, data, _, _) => {
                info!("OracleRequest: {}, {}, {:?}", operator_id, request_id, data);
                if &operator_id != signer.account_id() {
                    // request wasn't delegated to us so return
                    return Ok(());
                }

                // TODO spawn does not work for multiple calls
                // when I send 10 requests it only responds to one
                //tokio::spawn(async move {
                // TODO storage query
                // TODO verify user identities
                // TODO retrieve balances and check requirements
                let requirement_check = true;
                let result = vec![requirement_check as u8];

                let tx = oracle_callback(request_id, result);
                send_tx_ready(api, &tx, signer).await?;
                info!("oracle answer ({}) submitted", request_id);
                //   Ok::<(), anyhow::Error>(())
                //});
            }
        }
    }

    Ok(())
}
