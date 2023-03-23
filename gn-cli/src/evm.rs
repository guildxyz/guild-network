use ethers::core::k256::ecdsa::SigningKey;
use ethers::signers::{LocalWallet, Signer};
use gn_common::pad::pad_to_n_bytes;

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub fn generate_test_accounts(output: PathBuf, num: usize, seed: &str) {
    let mut file = File::create(output).expect("failed to create file");
    write!(&mut file, "{}", &format!("SEED:{}", seed)).expect("failed to write file");
    let mut seed_bytes = pad_to_n_bytes::<32, _>(seed);
    let mut index = 0usize;
    for _ in 0..num {
        let signing_key = SigningKey::from_bytes(&seed_bytes).expect("invalid seed");
        let signer = LocalWallet::from(signing_key);
        increment_seed(&mut seed_bytes, &mut index);
        write!(&mut file, "\n{:?}", signer.address()).expect("failed to write file");
    }
}

// panics if index is out of bounds
fn increment_seed(seed: &mut [u8; 32], index: &mut usize) {
    if seed[*index] == u8::MAX {
        *index += 1;
        increment_seed(seed, index)
    } else {
        seed[*index] += 1;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[should_panic]
    fn overflow() {
        let mut seed = [u8::MAX; 32];
        increment_seed(&mut seed, &mut 0);
    }

    #[test]
    fn seed_incremented_properly() {
        let mut seed = [0u8; 32];
        let mut index = 0usize;
        increment_seed(&mut seed, &mut index);
        assert_eq!(index, 0);
        assert_eq!(seed[index], 1);

        for _ in 1..u8::MAX {
            increment_seed(&mut seed, &mut index);
        }

        assert_eq!(index, 0);
        assert_eq!(seed[index], u8::MAX);

        increment_seed(&mut seed, &mut index);

        assert_eq!(index, 1);
        assert_eq!(seed[index], 1);
    }
}
