#[cfg(feature = "wallet")]
pub mod wallet;

use ecdsa::RecoveryId;
use gn_common::hash::sha2_256;
use p256::ecdsa::{Signature, VerifyingKey};

pub fn recover_prehashed(msg: &[u8; 32], signature: &[u8; 65]) -> Option<VerifyingKey> {
    let recid = RecoveryId::from_byte(signature[64])?;
    let sig = Signature::from_slice(&signature[0..64]).ok()?;
    VerifyingKey::recover_from_prehash(msg, &sig, recid).ok()
}

pub fn hash_pubkey(pubkey: &VerifyingKey) -> [u8; 32] {
    sha2_256(pubkey.to_encoded_point(false).as_bytes())
}

pub fn hash_account_id<T: parity_scale_codec::Encode>(account_id: T) -> [u8; 32] {
    sha2_256(account_id.encode())
}

#[cfg(test)]
mod test {
    use super::*;
    use base64::{engine::general_purpose, Engine as _};
    use p256::ecdsa::SigningKey;

    const MSG: &[u8] = b"almafa";
    const PRIVKEY: &str = "ABW2Jvg8gqn09yV2rvqYHihUNWL6kv6uzGq_WkF8_3w";
    const PUBKEY_X: &str = "zb6Gsg2U2zOrfcfpLjeCBMiS9M35YPgz-ceQdgsMPtc";
    const PUBKEY_Y: &str = "r-e3AGl9P-rThq5SiVyXupt11JGpBnxB59-WjtJVLnM";
    const HEX_PUBKEY: &str = "04cdbe86b20d94db33ab7dc7e92e378204c892f4cdf960f833f9c790760b0c3ed7afe7b700697d3fead386ae52895c97ba9b75d491a9067c41e7df968ed2552e73";
    const HEX_SIGNATURE: &str = "38bec2be55cfdbf6084db569246744c7559e5becd260fdaec6e67c560575805bc9b59c705173c937cd592db858068ccc5525953ca1d26bec0f1df16b867d8da501";

    #[test]
    fn webcrypto() {
        let privkey_bytes = general_purpose::URL_SAFE_NO_PAD.decode(PRIVKEY).unwrap();
        let pubkey_x_bytes = general_purpose::URL_SAFE_NO_PAD.decode(PUBKEY_X).unwrap();
        let pubkey_y_bytes = general_purpose::URL_SAFE_NO_PAD.decode(PUBKEY_Y).unwrap();

        let privkey = SigningKey::from_slice(&privkey_bytes).unwrap();
        let pubkey = VerifyingKey::from(&privkey);
        let serialized_pubkey = pubkey.to_encoded_point(false); // don't compress

        let hex_decoded_pubkey = hex::decode(HEX_PUBKEY).unwrap();
        let hex_decoded_signature: [u8; 65] =
            hex::decode(HEX_SIGNATURE).unwrap().try_into().unwrap();

        let recovered_pubkey = recover_prehashed(&sha2_256(MSG), &hex_decoded_signature).unwrap();

        assert_eq!(hex_decoded_pubkey.len(), 65);
        assert_eq!(hex_decoded_pubkey[0], 0x04);
        assert_eq!(hex_decoded_pubkey[1..33], pubkey_x_bytes);
        assert_eq!(hex_decoded_pubkey[33..], pubkey_y_bytes);

        assert_eq!(serialized_pubkey.as_bytes(), hex_decoded_pubkey.as_slice());
        assert_eq!(pubkey, recovered_pubkey);
        assert_eq!(hash_pubkey(&pubkey), hash_pubkey(&recovered_pubkey));
    }
}
