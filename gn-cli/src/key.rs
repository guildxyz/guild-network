#[cfg(feature = "verify")]
use gn_api::query;
use gn_api::{
    tx::{self, Keypair, PairT, Signer},
    Api, SessionKeys,
};
use parity_scale_codec::Decode;
#[cfg(feature = "verify")]
use parity_scale_codec::Encode;

use std::sync::Arc;

pub fn generate(curve: &str, password: &str) {
    match curve {
        "sr25519" => {
            let (keypair, phrase, seed) = Keypair::generate_with_phrase(Some(password));
            log::info!("{}", keypair.public());
            log::info!("{phrase}");
            log::info!("{seed:?}");
        }
        _ => unimplemented!(),
    }
}

pub async fn rotate(api: Api) -> Vec<u8> {
    let keys = api.rpc().rotate_keys().await.unwrap();
    log::info!("aura: 0x{}", hex::encode(&keys.0[0..32]));
    log::info!("gran: 0x{}", hex::encode(&keys.0[32..64]));
    keys.0
}

pub async fn set(api: Api, signer: Arc<Signer>, encoded_keys: Vec<u8>) {
    let keys = SessionKeys::decode(&mut &encoded_keys[..]).expect("invalid keys");
    let proof = Vec::new();
    let payload = tx::set_session_keys(keys, proof);

    #[cfg(not(feature = "verify"))]
    tx::send::ready(api.clone(), &payload, signer.clone())
        .await
        .expect(super::TX_ERROR);

    #[cfg(feature = "verify")]
    {
        tx::send::in_block(api.clone(), &payload, signer.clone())
            .await
            .expect(super::TX_ERROR);
        // NOTE needs to be decoded again because `SessionKeys` is imported via
        // the subxt macro into a dummy runtime module where `Clone` is not
        // implemented for `SessionKeys`
        let keys = SessionKeys::decode(&mut &encoded_keys[..]).expect("invalid keys");
        let on_chain_keys = query::next_session_keys(api, signer.account_id())
            .await
            .expect(super::QUERY_ERROR);

        assert_eq!(keys.encode(), on_chain_keys.encode());

        log::info!("session keys set successfully");
    }
}
