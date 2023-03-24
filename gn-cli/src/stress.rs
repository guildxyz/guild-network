use ethers::core::k256::ecdsa::SigningKey;
use ethers::signers::{LocalWallet, Signer};
use futures::future::try_join_all;
use gn_client::{
    tx::{self, TxStatus},
    Api,
};
use gn_common::identity::{Identity, IdentityWithAuth};
use gn_common::pad::pad_to_n_bytes;

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
        increment_array(&mut seed_bytes, &mut index);
        write!(&mut file, "\n{:?}", signer.address()).expect("failed to write file");
    }
}

pub async fn register_other_identity(api: Api, num: usize, seed: &str, tps: usize, id_index: u8) {
    let mut id = 0usize;
    let mut seed_bytes = pad_to_n_bytes::<32, _>(seed);
    let mut index = 0usize;
    for (i, chunk_start) in (0..num).step_by(tps).enumerate() {
        let chunk = chunk_start..(chunk_start + tps).min(num);
        let tx_futures = chunk
            .map(|_| {
                let password = format!("//{}", hex::encode(&seed_bytes));
                let signer = tx::signer("", Some(&password)).expect("invalid signer");
                let identity = Identity::Other(pad_to_n_bytes::<64, _>(format!("other{}", id)));
                let identity_with_auth = IdentityWithAuth::Other(identity, [0u8; 64]);
                id += 1;
                increment_array(&mut seed_bytes, &mut index);
                let payload = tx::register(identity_with_auth, id_index);
                tx::send::owned(api.clone(), payload, signer, TxStatus::Ready)
            })
            .collect::<Vec<_>>();

        try_join_all(tx_futures)
            .await
            .expect("failed to send transactions");

        tokio::time::sleep(std::time::Duration::from_millis(950)).await;
        log::info!("registration batch {} submitted", i);
    }
}

// panics if index is out of bounds
fn increment_array(array: &mut [u8], index: &mut usize) {
    if array[*index] == u8::MAX {
        *index += 1;
        increment_array(array, index)
    } else {
        array[*index] += 1;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[should_panic]
    fn overflow() {
        let mut seed = [u8::MAX; 32];
        increment_array(&mut seed, &mut 0);
    }

    #[test]
    fn array_incremented_properly() {
        let mut seed = [0u8; 32];
        let mut index = 0usize;
        increment_array(&mut seed, &mut index);
        assert_eq!(index, 0);
        assert_eq!(seed[index], 1);

        for _ in 1..u8::MAX {
            increment_array(&mut seed, &mut index);
        }

        assert_eq!(index, 0);
        assert_eq!(seed[index], u8::MAX);

        increment_array(&mut seed, &mut index);

        assert_eq!(index, 1);
        assert_eq!(seed[index], 1);
    }
}
