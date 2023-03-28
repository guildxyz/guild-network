use super::TX_ERROR;
use gn_api::{
    tx::{self, Signer},
    AccountId, Api,
};

use std::str::FromStr;
use std::sync::Arc;

pub async fn transfer(api: Api, signer: Arc<Signer>, id_str: &str, balance: u128) {
    let account_id = AccountId::from_str(id_str).expect("invalid account id string");
    let payload = tx::transfer(&account_id, balance);
    tx::send::in_block(api, &payload, signer)
        .await
        .expect(TX_ERROR);
    log::info!("{account_id} received {balance} tokens");
}
