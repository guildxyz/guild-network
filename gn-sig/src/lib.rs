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
