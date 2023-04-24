use gn_common::hash::sha2_256;
use p256::ecdsa::{SigningKey, VerifyingKey};

pub struct Wallet(SigningKey);

impl Wallet {
    pub fn from_seed(seed: [u8; 32]) -> Option<Self> {
        SigningKey::from_slice(&seed).ok().map(|k| Self(k))
    }

    pub fn sign<T: AsRef<[u8]>>(&self, msg: T) -> Option<[u8; 65]> {
        self.0
            .sign_prehash_recoverable(&sha2_256(msg))
            .ok()
            .map(|(sig, rec_id)| {
                let mut sig_bytes = [0u8; 65];
                sig_bytes[..64].copy_from_slice(&sig.to_bytes());
                sig_bytes[64] = rec_id.to_byte();
                sig_bytes
            })
    }

    pub fn pubkey(&self) -> VerifyingKey {
        VerifyingKey::from(&self.0)
    }
}

#[cfg(test)]
mod test {
    use super::super::recover_prehashed;
    use super::*;

    #[test]
    fn signature() {
        let msg = "this a long message that's at least 32 bytes long";
        let wallet = Wallet::from_seed([32u8; 32]).unwrap();
        let signature = wallet.sign(msg).unwrap();

        let recovered_pubkey = recover_prehashed(&sha2_256(msg), &signature).unwrap();
        assert_eq!(recovered_pubkey, wallet.pubkey());
    }
}
