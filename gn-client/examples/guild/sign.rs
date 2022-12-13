use gn_client::queries;
use gn_client::transactions::{track_progress, register_operator, TxStatus};
use gn_client::{Api, Signer, TxSignerTrait};

use std::sync::Arc;

pub async fn sign(api: Api, alice: Arc<Signer>) {
    let prepared = api
        .tx()
        .prepare_unsigned(
            &register_operator(),
            alice.as_ref().account_id(),
            Default::default(),
        )
        .await
        .expect("failed to prepare msg");

    let signature = alice.as_ref().sign(&prepared.prepared_msg);

    println!(
        "ADDRESS: {}, ID: {}",
        alice.as_ref().account_id(),
        alice.as_ref().address()
    );

    let mut progress = api
        .tx()
        .pack_and_submit_then_watch(
            alice.as_ref().address(),
            signature,
            &prepared.encoded_params,
        )
        .await
        .expect("failed to submit extrisic");

    track_progress(&mut progress, TxStatus::InBlock)
        .await
        .expect("failed to track status");

    let registered_operators = queries::registered_operators(api.clone())
        .await
        .expect("failed to fetch registered operators");
    // check that the transaction was successful
    assert_eq!(&registered_operators[0], alice.as_ref().account_id());
}
