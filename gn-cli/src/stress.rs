use ethers::core::k256::ecdsa::SigningKey;
use ethers::signers::{LocalWallet, Signer};
use futures::future::try_join_all;
use gn_api::{tx, Api};
use gn_common::identity::{Identity, IdentityWithAuth};
use gn_common::pad::pad_to_n_bytes;
use itertools::Itertools;
use std::time::Duration;

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub fn generate_evm_addresses(output: PathBuf, num: usize, seed: &str) {
    let mut file = File::create(output).expect("failed to create file");
    write!(&mut file, "{}", &format!("SEED:{}", seed)).expect("failed to write file");
    let mut seed_bytes = pad_to_n_bytes::<32, _>(seed);
    let mut index = 0usize;
    for _ in 0..num {
        let signing_key = SigningKey::from_bytes(&seed_bytes).expect("invalid seed");
        let signer = LocalWallet::from(signing_key);
        seed_bytes[index] = seed_bytes[index].wrapping_add(1);
        index = index.wrapping_add(1) % seed_bytes.len();
        write!(&mut file, "\n{:?}", signer.address()).expect("failed to write file");
    }
}

pub async fn register_other_identity(api: Api, num: usize, seed: &str, tps: usize, id_index: u8) {
    let mut id = 0usize;
    let mut index = 0usize;
    let mut seed_bytes = pad_to_n_bytes::<32, _>(seed);

    for chunk in &(0..num).chunks(tps) {
        let tx_futures = chunk
            .map(|_i| {
                let password = format!("//{}", hex::encode(seed_bytes));
                let signer = tx::signer("", Some(&password)).expect("invalid signer");
                let identity = Identity::Other(pad_to_n_bytes::<64, _>(format!("other{}", id)));
                let identity_with_auth = IdentityWithAuth::Other(identity, [0u8; 64]);
                id += 1;
                seed_bytes[index] = seed_bytes[index].wrapping_add(1);
                index = index.wrapping_add(1) % seed_bytes.len();
                let payload = tx::register(identity_with_auth, id_index);
                tx::send::owned_but_dont_watch(api.clone(), payload, signer)
            })
            .collect::<Vec<_>>();
        try_join_all(tx_futures).await.expect("failed to send tx");
        tokio::time::sleep(Duration::from_secs(1)).await;
        log::info!("registration batch submitted");
    }
}
