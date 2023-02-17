use gn_client::{tx::Signer, Api};
use std::sync::Arc;

pub async fn keys(api: Api, _alice: Arc<Signer>) {
    let keys = api.rpc().rotate_keys().await.unwrap();
    println!("{:?}", keys);
    println!("{}", keys.len());
    println!("{}", hex::encode(keys.0))
}
