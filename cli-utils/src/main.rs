use codec::Encode;
use futures::future::try_join_all;
use log::{error, info, warn};
use sp_core::crypto::Pair as TraitPair;
use sp_core::H256 as TxHash;
use sp_keyring::sr25519::sr25519::Pair as Keypair;
use sp_keyring::AccountKeyring;
use subxt::ext::sp_runtime::MultiAddress;
use subxt::tx::PairSigner;
use subxt::{OnlineClient, PolkadotConfig};

use std::sync::Arc;

const URL: &str = "ws://127.0.0.1:9944";
type Api = OnlineClient<PolkadotConfig>;

#[subxt::subxt(runtime_metadata_path = "./artifacts/metadata.scale")]
pub mod node {}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let api = Api::from_url(URL)
        .await
        .expect("failed to initialize client");
    let faucet = Arc::new(PairSigner::new(AccountKeyring::Alice.pair()));

    // generate new keypairs
    let mut seed = [10u8; 32];
    let operators = (0..10)
        .map(|_| {
            let keypair = PairSigner::new(Keypair::from_seed(&seed));
            seed[0] += 1;
            keypair
        })
        .collect::<Vec<PairSigner<PolkadotConfig, Keypair>>>();

    let amount = 100_000u128;
    let fund_futures = operators
        .iter()
        .map(|operator| {
            fund_account(
                api.clone(),
                Arc::clone(&faucet),
                operator.account_id(),
                amount,
            )
        })
        .collect::<Vec<_>>();

    try_join_all(fund_futures)
        .await
        .expect("failed to fund accounts");
}

async fn register_operator(api: Api) -> Result<(), anyhow::Error> {
    todo!();
}

async fn fund_account(
    api: Api,
    from: Arc<PairSigner<PolkadotConfig, Keypair>>,
    to: &sp_core::crypto::AccountId32,
    amount: u128,
) -> Result<TxHash, subxt::Error> {
    let tx = node::tx()
        .balances()
        .transfer(MultiAddress::Id(to.clone()), amount);
    api.tx().sign_and_submit_default(&tx, from.as_ref()).await
}

async fn create_guild() {
    todo!();
}

async fn join_guild() {
    todo!();
}
