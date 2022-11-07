use futures::future::try_join_all;
use futures::StreamExt;
use guild_network_client::{
    api, AccountId, Api, BlockSubscription, Keypair, Signer, TransactionProgress,
};
use guild_network_gate::requirements::Requirement;
use sp_keyring::AccountKeyring;
use subxt::ext::sp_core::crypto::Pair as TraitPair;
use subxt::ext::sp_core::H256 as TxHash;
use subxt::ext::sp_runtime::MultiAddress;
use subxt::tx::TxStatus;

use std::sync::Arc;

const URL: &str = "ws://127.0.0.1:9944";

#[tokio::main]
async fn main() {
    let api = Api::from_url(URL)
        .await
        .expect("failed to initialize client");
    let faucet = Arc::new(Signer::new(AccountKeyring::Alice.pair()));

    let mut blocks: BlockSubscription = api
        .rpc()
        .subscribe_blocks()
        .await
        .expect("failed to subscribe to blocks");

    // generate new keypairs
    let mut seed = [10u8; 32];
    let operators = (0..10)
        .map(|_| {
            let keypair = Arc::new(Signer::new(Keypair::from_seed(&seed)));
            seed[0] += 1;
            keypair
        })
        .inspect(|x| println!("{}", x.as_ref().account_id()))
        .collect::<Vec<Arc<Signer>>>();

    let amount = 1_000_000_000_000_000u128;
    // NOTE sending a TX with the same signer too quickly will result in
    // a priority too low error (because the nonce will be the same for some reason)
    /*
    let fund_futures = operators
        .iter()
        .map(|operator| {
            fund_account(
                api.clone(),
                Arc::clone(&faucet),
                operator.as_ref().account_id(),
                amount,
            )
        })
        .collect::<Vec<_>>();

    try_join_all(fund_futures)
        .await
        .expect("failed to fund accounts");
    */

    // NOTE this looks horrible but we need a way to wait for the
    // balances transfers, so we get a tx progress handle to the
    // last tx in the queue
    let mut last_tx: Option<TransactionProgress> = None;
    for operator in &operators {
        last_tx = Some(
            fund_account(
                api.clone(),
                Arc::clone(&faucet),
                operator.account_id(),
                amount,
            )
            .await
            .expect("failed to fund account"),
        )
    }

    let mut last_tx = last_tx.unwrap();

    // NOTE we have to wait for the funds to arrive (i.e. transfers need to finalize)
    while let Some(event_info) = last_tx.next().await {
        let event = event_info.unwrap();
        if let TxStatus::InBlock(_) = event {
            println!("Balance transfers included in block");
            break;
        }
    }

    let block_number = blocks.next().await.unwrap().unwrap().number + 1;

    let register_operator_futures = operators
        .iter()
        .map(|operator| register_operator(api.clone(), Arc::clone(operator)))
        .collect::<Vec<_>>();

    try_join_all(register_operator_futures)
        .await
        .expect("failed to register operators");

    // wait for next block
    while block_number >= blocks.next().await.unwrap().unwrap().number {}

    let registered_operators = api::storage().chainlink().operators();
    let on_chain_operators = api
        .storage()
        .fetch(&registered_operators, None)
        .await
        .unwrap()
        .unwrap();
    for operator in &on_chain_operators {
        println!("{}", operator);
    }
}

async fn register_operator(api: Api, signer: Arc<Signer>) -> Result<TxHash, subxt::Error> {
    let tx = api::tx().chainlink().register_operator();
    api.tx().sign_and_submit_default(&tx, signer.as_ref()).await
}

async fn fund_account(
    api: Api,
    from: Arc<Signer>,
    to: &AccountId,
    amount: u128,
) -> Result<TransactionProgress, subxt::Error> {
    let tx = api::tx()
        .balances()
        .transfer(MultiAddress::Id(to.clone()), amount);
    api.tx()
        .sign_and_submit_then_watch_default(&tx, from.as_ref())
        .await
}

async fn create_guild(api: Api, signer: Arc<Signer>, guild: Guild) {}

async fn join_guild() {
    todo!();
}

fn pad_to_32_bytes(name: &[u8]) -> Result<[u8; 32], anyhow::Error> {
    let mut output = [0u8; 32];
    anyhow::ensure!(name.len() <= output.len(), "name too long");
    output.copy_from_slice(name);
    Ok(output)
}

struct Guild {
    name: String,
    metadata: Vec<u8>,
    roles: Vec<Role>,
}

struct Role {
    name: String,
    requirements: Vec<Requirement>,
}
