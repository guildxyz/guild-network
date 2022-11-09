use futures::future::try_join_all;
use guild_network_client::queries::*;
use guild_network_client::transactions::*;
use guild_network_client::{Api, Keypair, Signer, TxStatus};
use sp_keyring::AccountKeyring;
use subxt::ext::sp_core::crypto::Pair as TraitPair;

use std::sync::Arc;

const URL: &str = "ws://127.0.0.1:9944";

#[tokio::main]
async fn main() {
    let api = Api::from_url(URL)
        .await
        .expect("failed to initialize client");
    let faucet = Arc::new(Signer::new(AccountKeyring::Alice.pair()));

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

    for operator in operators.iter().skip(1) {
        // skip first
        let tx = fund_account(operator.account_id(), amount);
        send_tx(api.clone(), tx, Arc::clone(&faucet), TxStatus::Ready)
            .await
            .unwrap();
    }
    // wait for the skipped one to be included in a block
    let tx = fund_account(operators[0].account_id(), amount);
    send_tx(api.clone(), tx, Arc::clone(&faucet), TxStatus::InBlock)
        .await
        .unwrap();

    println!("Balance transfers in block!");

    let register_operator_futures = operators
        .iter()
        .map(|operator| {
            let tx = register_operator();
            send_tx(api.clone(), tx, Arc::clone(operator), TxStatus::InBlock)
        })
        .collect::<Vec<_>>();

    try_join_all(register_operator_futures)
        .await
        .expect("failed to register operators");

    println!("Operator registrations in block!");

    let registered_operators = registered_operators(api).await.unwrap();

    for operator in &registered_operators {
        println!("{}", operator);
    }
}
