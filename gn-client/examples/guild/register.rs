use gn_client::{
    tx::{self, Signer},
    Api,
};

use std::sync::Arc;

pub async fn register(api: Api, root: Arc<Signer>) {
    let payload = tx::register_operator(root.account_id());
    tx::send_tx_in_block(api, &tx::sudo(payload), root)
        .await
        .unwrap();
    println!("root registered as operator");
}
