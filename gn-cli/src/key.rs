use gn_client::{
    query,
    tx::{self, Keypair, PairT, Signer},
    Api, SessionKeys,
};
use parity_scale_codec::{Decode, Encode};

use std::sync::Arc;

pub fn generate(curve: &str, password: Option<&str>) {
    match curve {
        "sr25519" => {
            let (keypair, phrase, seed) = Keypair::generate_with_phrase(password);
            println!("{}", keypair.public());
            println!("{phrase}");
            println!("{seed:?}");
        }
        _ => unimplemented!(),
    }
}

pub async fn rotate(api: Api) -> Vec<u8> {
    let keys = api.rpc().rotate_keys().await.unwrap();
    println!("aura: 0x{}", hex::encode(&keys.0[0..32]));
    println!("gran: 0x{}", hex::encode(&keys.0[32..64]));
    keys.0
}

pub async fn set(api: Api, signer: Arc<Signer>, encoded_keys: Vec<u8>) {
    let keys = SessionKeys::decode(&mut &encoded_keys[..]).expect("invalid keys");
    let proof = Vec::new();
    let payload = tx::set_session_keys(keys, proof);

    tx::send_tx_in_block(api.clone(), &payload, signer.clone())
        .await
        .expect("failed to send tx");

    let keys = SessionKeys::decode(&mut &encoded_keys[..]).expect("invalid keys");
    let on_chain_keys = query::next_session_keys(api, signer.account_id())
        .await
        .expect("failed to query session keys");

    assert_eq!(keys.encode(), on_chain_keys.encode());

    println!("session keys set successfully");
}
