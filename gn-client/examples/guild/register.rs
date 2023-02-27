use gn_client::{
    tx::{self, Signer},
    AccountId, Api,
};

use std::str::FromStr;
use std::sync::Arc;

pub async fn register(api: Api, root: Arc<Signer>, maybe_operator: Option<&str>) {
    let account_id = if let Some(operator) = maybe_operator {
        AccountId::from_str(operator).expect("invalid operator id string")
    } else {
        root.account_id().clone()
    };

    let payload = tx::register_operator(&account_id);
    tx::send_tx_in_block(api, &tx::sudo(payload), root)
        .await
        .unwrap();
    println!("{account_id} registered as operator");
}
