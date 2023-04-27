use gn_common::hash::keccak256;
use secp256k1::{
    ecdsa::{RecoverableSignature, RecoveryId},
    Message, PublicKey, Secp256k1,
};

const ETHEREUM_HASH_PREFIX: &str = "\x19Ethereum Signed Message:\n";
pub const EVM_ADDRESS_PREFIX: &[u8] = b"evm-address:";

pub fn pubkey2address(pubkey: &PublicKey) -> [u8; 20] {
    let serialized_pk = pubkey.serialize_uncompressed();
    debug_assert_eq!(serialized_pk[0], 0x04);
    keccak256(&serialized_pk[1..])[12..]
        .try_into()
        .expect("address is 20 bytes; qed")
}

pub fn address2account(evm_address: [u8; 20]) -> [u8; 32] {
    let mut address = [0u8; 32];
    address[0..12].copy_from_slice(EVM_ADDRESS_PREFIX);
    address[12..].copy_from_slice(&evm_address);
    address
}

pub fn pubkey2account(pubkey: &PublicKey) -> [u8; 32] {
    address2account(pubkey2address(pubkey))
}

pub fn hash_message<M: AsRef<[u8]>>(message: M) -> [u8; 32] {
    let mut eth_message =
        scale_info::prelude::format!("{ETHEREUM_HASH_PREFIX}{}", message.as_ref().len())
            .into_bytes();
    eth_message.extend_from_slice(message.as_ref());
    keccak256(&eth_message)
}

/// Recovers the signer's public key from a pre-hashed message and the provided
/// signature.
///
/// In case of an invalid signature, the function returns None. It is important
/// that the recovery id of the signature (the last byte) is normalized to
/// either 0 or 1.
pub fn recover_prehashed(
    message: &[u8; 32],
    signature: &crate::EcdsaSignature,
) -> Option<PublicKey> {
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
    use super::{hash_message, pubkey2account, pubkey2address, recover_prehashed};
    use ethers::core::k256::ecdsa::SigningKey;
    use ethers::signers::{LocalWallet, Signer as SignerT};

    const MSG: &str = "hello from guild network";

    #[test]
    fn test_eth_msg_hashing() {
        let eth_hashed_msg = hash_message(MSG);
        assert_eq!(ethers::utils::hash_message(MSG).as_bytes(), eth_hashed_msg);
    }

    #[test]
    fn ethereum_ecdsa() {
        let seed = [2u8; 32];
        let signing_key = SigningKey::from_bytes(&seed).unwrap();
        let signer = LocalWallet::from(signing_key);

        let address_bytes = signer.address().to_fixed_bytes();
        let hashed_msg = hash_message(MSG);

        let mut expected_account = [0u8; 32];
        expected_account[0..12].copy_from_slice(b"evm-address:");
        expected_account[12..].copy_from_slice(&address_bytes);

        // generate signature
        let mut signature: [u8; 65] = futures::executor::block_on(async move {
            signer
                .sign_message(MSG)
                .await
                .unwrap()
                .to_vec()
                .try_into()
                .unwrap()
        });

        // normalize recovery id due to eip-155
        signature[64] -= 27;

        // recover signer's public key
        let recovered_key = recover_prehashed(&hashed_msg, &signature).unwrap();

        // check key validity
        assert_eq!(pubkey2address(&recovered_key), address_bytes);
        assert_eq!(pubkey2account(&recovered_key), expected_account);
    }
}
