use codec::Encode;
use futures::future::try_join_all;
use log::{error, info, warn};
use sp_core::crypto::Pair as TraitPair;
use sp_keyring::sr25519::sr25519::Pair as Keypair;
use sp_keyring::AccountKeyring;
use subxt::tx::PairSigner;
use subxt::{OnlineClient, PolkadotConfig};

use std::sync::Arc;

const URL: &str = "ws://127.0.0.1:9944";
type Client = OnlineClient<PolkadotConfig>;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let client = Client::from_url(URL)
        .await
        .expect("failed to initialize client");
    let faucet = Arc::new(PairSigner::new(AccountKeyring::Alice.pair()));

    // generate new keypairs
    let mut seed = [10u8; 32];
    let operators = (0..10)
        .map(|_| {
            let keypair = Keypair::from_seed(&seed);
            seed[0] += 1;
            keypair
        })
        .collect::<Vec<Keypair>>();

    let amount = 100_000u128;
    let fund_futures = operators
        .iter()
        .map(|operator| {
            fund_account(
                client.clone(),
                Arc::clone(&faucet),
                operator.public(),
                amount,
            )
        })
        .collect::<Vec<_>>();

    try_join_all(fund_futures)
        .await
        .expect("failed to fund accounts");
}

async fn register_operator(client: Client) -> Result<(), anyhow::Error> {
    todo!();
}

async fn fund_account(
    client: Client,
    from: Arc<PairSigner<PolkadotConfig, Keypair>>,
    to: <Keypair as TraitPair>::Public,
    amount: u128,
) -> Result<(), anyhow::Error> {
    todo!();
}

async fn create_guild() {
    todo!();
}

async fn join_guild() {
    todo!();
}
