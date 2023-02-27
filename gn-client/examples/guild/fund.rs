use gn_client::{
    tx::{self, Signer},
    AccountId, Api,
};

use std::str::FromStr;
use std::sync::Arc;

pub async fn fund(api: Api, signer: Arc<Signer>, id_str: &str, balance: u128) {
    let account_id = AccountId::from_str(id_str).expect("invalid account id string");
    let payload = tx::fund_account(&account_id, balance);
    tx::send_tx_in_block(api, &payload, signer).await.unwrap();
    println!("{account_id} received {balance} tokens");
}
