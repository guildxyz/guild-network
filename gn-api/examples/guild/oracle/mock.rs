use super::*;
use futures::future::try_join_all;
use gn_api::tx::{self, Keypair, PairT, TxStatus};

use std::sync::Arc;

pub async fn init_dummy_operators(api: Api, root: Arc<Signer>) -> Vec<Arc<Signer>> {
    let operators = dummy_operators();
    register_operators(api.clone(), root, &operators).await;
    wait_for_registered_operator(api.clone(), operators[0].account_id()).await;
    activate_operators(api, &operators).await;
    operators
}

fn dummy_operators() -> Vec<Arc<Signer>> {
    let mut seed = ACCOUNT_SEED;
    (0..N_TEST_ACCOUNTS)
        .map(|_| {
            let keypair = Arc::new(Signer::new(Keypair::from_seed(&seed)));
            seed[0] += 1;
            keypair
        })
        .inspect(|acc| println!("new operator: {}", acc.account_id()))
        .collect()
}

async fn register_operators(api: Api, root: Arc<Signer>, accounts: &[Arc<Signer>]) {
    let payloads = accounts
        .iter()
        .map(|account| tx::sudo(tx::register_operator(account.as_ref().account_id())))
        .collect::<Vec<tx::TxPayload>>();

    tx::send::batch(api, payloads.iter(), root)
        .await
        .expect("failed to send batch tx");

    println!("operator registrations submitted");
}

async fn activate_operators(api: Api, accounts: &[Arc<Signer>]) {
    println!("activating operators");
    let tx_futures = accounts
        .iter()
        .map(|acc| {
            tx::send::owned(
                api.clone(),
                tx::activate_operator(),
                Arc::clone(acc),
                TxStatus::InBlock,
            )
        })
        .collect::<Vec<_>>();

    try_join_all(tx_futures).await.unwrap();

    println!("operators activated");
}

async fn wait_for_registered_operator(api: Api, operator: &AccountId) {
    let mut i = 0;
    loop {
        if query::is_operator_registered(api.clone(), operator)
            .await
            .expect("failed to fetch registered operator")
        {
            break;
        }
        i += 1;
        println!("waiting for registered operators");
        if i == RETRIES {
            panic!("no registered operators found");
        }
        tokio::time::sleep(std::time::Duration::from_millis(SLEEP_DURATION_MS)).await;
    }
}

pub async fn send_dummy_oracle_answers(api: Api, operators: &[Arc<Signer>]) {
    let oracle_requests = query::oracle_requests(api.clone(), PAGE_SIZE)
        .await
        .expect("failed to fetch oracle requests");

    for (request_id, operator_id) in oracle_requests {
        let tx = tx::oracle_callback(request_id, vec![u8::from(true)]);
        let signer = operators
            .iter()
            .find(|operator| operator.account_id() == &operator_id)
            .unwrap();
        tx::send::ready(api.clone(), &tx, Arc::clone(signer))
            .await
            .expect("failed to submit oracle answer");
    }

    println!("oracle requests successfully answered");
}
