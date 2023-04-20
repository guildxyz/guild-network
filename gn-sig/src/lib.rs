#![cfg_attr(not(feature = "std"), no_std)]
#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(unused_crate_dependencies)]

mod eth;
mod multisignature;
mod multisigner;

pub use multisignature::MultiSignature;
pub use multisigner::MultiSigner;

use secp256k1::{
    ecdsa::{RecoverableSignature, RecoveryId},
    Message, PublicKey, Secp256k1,
};

/// Recovers the signer's public key from a pre-hashed message and the provided
/// signature.
///
/// In case of an invalid signature, the function returns None. It is important
/// that the recovery id of the signature (the last byte) is normalized to
/// either 0 or 1.
pub fn recover_prehashed(message: &[u8; 32], signature: &[u8; 65]) -> Option<PublicKey> {
    let rid = RecoveryId::from_i32(signature[64] as i32).ok()?;
    let sig = RecoverableSignature::from_compact(&signature[..64], rid).ok()?;
    // NOTE this never fails because the prehashed message is 32 bytes
    let message = Message::from_slice(message).expect("Message is 32 bytes; qed");
    Secp256k1::verification_only()
        .recover_ecdsa(&message, &sig)
        .ok()
}

#[cfg(test)]
mod test {
    use super::*;
    use secp256k1::SecretKey;
    use base64::{Engine as _, engine::general_purpose};

    const MSG: &str = "almafa";
    const PRIVKEY: &str = "ABW2Jvg8gqn09yV2rvqYHihUNWL6kv6uzGq_WkF8_3w";
    const PUBKEY_X: &str = "zb6Gsg2U2zOrfcfpLjeCBMiS9M35YPgz-ceQdgsMPtc";
    const PUBKEY_Y: &str = "r-e3AGl9P-rThq5SiVyXupt11JGpBnxB59-WjtJVLnM";
    const HEX_PUBKEY: &str = "04cdbe86b20d94db33ab7dc7e92e378204c892f4cdf960f833f9c790760b0c3ed7afe7b700697d3fead386ae52895c97ba9b75d491a9067c41e7df968ed2552e73";
    const HEX_SIGNATURE: &str = "38bec2be55cfdbf6084db569246744c7559e5becd260fdaec6e67c560575805bc9b59c705173c937cd592db858068ccc5525953ca1d26bec0f1df16b867d8da5";

    #[test]
    fn webcrypto() {
        let secp = Secp256k1::new();
        let privkey_bytes = general_purpose::URL_SAFE_NO_PAD.decode(PRIVKEY).unwrap();
        let pubkey_x_bytes = general_purpose::URL_SAFE_NO_PAD.decode(PUBKEY_X).unwrap();
        let pubkey_y_bytes = general_purpose::URL_SAFE_NO_PAD.decode(PUBKEY_Y).unwrap();

        let privkey = SecretKey::from_slice(&privkey_bytes).unwrap();
        let pubkey = privkey.public_key(&secp);
        let serialized_pubkey = pubkey.serialize_uncompressed();
        let hex_decoded_pubkey = hex::decode(HEX_PUBKEY).unwrap();
        assert_eq!(hex_decoded_pubkey.len(), 65);
        assert_eq!(hex_decoded_pubkey[0], 0x04);
        assert_eq!(hex_decoded_pubkey[1..33], pubkey_x_bytes);
        assert_eq!(hex_decoded_pubkey[33..], pubkey_y_bytes);

        assert_eq!(&serialized_pubkey, hex_decoded_pubkey.as_slice());
    }
}
