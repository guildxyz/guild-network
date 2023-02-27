use gn_client::{
    tx::{self, Signer},
    AccountId, Api,
};

use std::str::FromStr;
use std::sync::Arc;

pub async fn register(api: Api, root: Arc<Signer>, operator: &str) {
    let account_id = AccountId::from_str(operator).expect("invalid operator id string");
    let payload = tx::register_operator(&account_id);
    tx::send_tx_in_block(api, &tx::sudo(payload), root)
        .await
        .unwrap();
    println!("{operator} registered as operator");
}
