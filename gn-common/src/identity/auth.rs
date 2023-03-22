use super::Identity;
use crate::hash::keccak256;
use crate::{Decode, Encode, TypeInfo};
use ed25519_zebra::{Signature as EdSig, VerificationKey as EdKey};
use schnorrkel::{PublicKey as SrKey, Signature as SrSig};
use secp256k1::{
    ecdsa::{RecoverableSignature, RecoveryId},
    Message, Secp256k1,
};

const ETHEREUM_HASH_PREFIX: &str = "\x19Ethereum Signed Message:\n";
const SR_SIGNING_CTX: &[u8] = b"substrate";

#[derive(Encode, Decode, TypeInfo, Eq, PartialEq, Clone, Copy, Debug)]
pub struct EcdsaSignature(pub [u8; 65]);
#[derive(Encode, Decode, TypeInfo, Eq, PartialEq, Clone, Copy, Debug)]
pub struct Ed25519Signature(pub [u8; 64]);
#[derive(Encode, Decode, TypeInfo, Eq, PartialEq, Clone, Copy, Debug)]
pub struct Sr25519Signature(pub [u8; 64]);

#[derive(Encode, Decode, TypeInfo, Eq, PartialEq, Clone, Copy, Debug)]
pub enum IdentityWithAuth {
    Ecdsa(Identity, EcdsaSignature),
    Ed25519(Identity, Ed25519Signature),
    Sr25519(Identity, Sr25519Signature),
    Other(Identity, [u8; 64]),
}

impl IdentityWithAuth {
    pub fn verify<M: AsRef<[u8]>>(&self, msg: M) -> bool {
        match self {
            // Ethereum specific ecdsa
            Self::Ecdsa(Identity::Address20(address), sig) => {
                let prehashed_msg = eth_hash_message(msg);
                let Some(recovered_pk) = recover_prehashed(prehashed_msg, sig) else {
                    return false
                };

                let serialized_pk = recovered_pk.serialize_uncompressed();
                debug_assert_eq!(serialized_pk[0], 0x04);
                &keccak256(&serialized_pk[1..])[12..] == address
            }
            // generic ecdsa - only works with prehashed messages because
            // everyone might choose different hashing algorithms
            Self::Ecdsa(Identity::Address32(address), sig) => {
                let Ok(prehashed_msg) = msg.as_ref().try_into() else {
                    return false
                };
                let Some(recovered_pk) = recover_prehashed(prehashed_msg,  sig) else {
                    return false
                };
                &recovered_pk.serialize()[1..] == address
            }
            Self::Ed25519(Identity::Address32(pubkey), sig) => {
                let Ok(ed_key) = EdKey::try_from(pubkey.as_ref()) else {
                    return false
                };

                let Ok(ed_sig) = EdSig::try_from(&sig.0[..]) else {
                    return false
                };

                ed_key.verify(&ed_sig, msg.as_ref()).is_ok()
            }
            Self::Sr25519(Identity::Address32(pubkey), sig) => {
                let Ok(sr_key) = SrKey::from_bytes(pubkey.as_ref()) else {
                    return false
                };

                let Ok(sr_sig) = SrSig::from_bytes(&sig.0) else {
                    return false
                };

                sr_key
                    .verify_simple(SR_SIGNING_CTX, msg.as_ref(), &sr_sig)
                    .is_ok()
            }
            Self::Other(Identity::Other(_), _) => true,
            _ => false,
        }
    }

    pub fn from_evm(address: &str, signature: &str) -> Result<Self, &'static str> {
        let mut address20_bytes = [0u8; 20];
        hex::decode_to_slice(address.trim_start_matches("0x"), &mut address20_bytes)
            .map_err(|_| "invalid address")?;

        let mut signature_bytes = [0u8; 65];
        hex::decode_to_slice(signature.trim_start_matches("0x"), &mut signature_bytes)
            .map_err(|_| "invalid signature")?;

        // EIP-155 correction
        if signature_bytes[64] >= 27 {
            signature_bytes[64] -= 27;
        }

        Ok(IdentityWithAuth::Ecdsa(
            Identity::Address20(address20_bytes),
            EcdsaSignature(signature_bytes),
        ))
    }
}

pub fn recover_prehashed(
    message: [u8; 32],
    signature: &EcdsaSignature,
) -> Option<secp256k1::PublicKey> {
    let rid = RecoveryId::from_i32(signature.0[64] as i32).ok()?;
    let sig = RecoverableSignature::from_compact(&signature.0[..64], rid).ok()?;
    // NOTE this never fails because the prehashed message is 32 bytes
    let message = Message::from_slice(&message).expect("Message is 32 bytes; qed");
    Secp256k1::verification_only()
        .recover_ecdsa(&message, &sig)
        .ok()
}

pub fn eth_hash_message<M: AsRef<[u8]>>(message: M) -> [u8; 32] {
    let mut eth_message =
        scale_info::prelude::format!("{ETHEREUM_HASH_PREFIX}{}", message.as_ref().len())
            .into_bytes();
    eth_message.extend_from_slice(message.as_ref());
    keccak256(&eth_message)
}

#[cfg(any(test, feature = "test-sig"))]
pub fn sign_prehashed(seed: [u8; 32], message: [u8; 32]) -> EcdsaSignature {
    let secret = secp256k1::SecretKey::from_slice(&seed).expect("seed is 32 bytes; qed");
    let message = Message::from_slice(&message).expect("Message is 32 bytes; qed");

    let (rec_id, sig) = Secp256k1::signing_only()
        .sign_ecdsa_recoverable(&message, &secret)
        .serialize_compact();
    let mut sig_bytes = [0u8; 65];
    sig_bytes[64] = rec_id.to_i32() as u8;
    sig_bytes[0..64].copy_from_slice(&sig);
    EcdsaSignature(sig_bytes)
}

#[cfg(any(test, feature = "test-sig"))]
pub fn test_ecdsa_id_with_auth<M: AsRef<[u8]>>(
    seed: [u8; 32],
    msg: M,
) -> (Identity, EcdsaSignature) {
    let prehashed = eth_hash_message(msg);
    let signature = sign_prehashed(seed, prehashed);
    let pubkey = recover_prehashed(prehashed, &signature).unwrap();
    let address: [u8; 20] = keccak256(&pubkey.serialize_uncompressed()[1..])[12..]
        .try_into()
        .unwrap();
    (Identity::Address20(address), signature)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::verification_msg;
    use ethers::core::k256::ecdsa::SigningKey;
    use ethers::core::k256::elliptic_curve::sec1::ToEncodedPoint;
    use ethers::core::k256::elliptic_curve::PublicKey;
    use ethers::signers::{LocalWallet, Signer as EthSigner};
    use sha3::Digest;
    use sp_core::Pair as PairT;

    const TEST_ACCOUNT: &str = "test-account-0xabcde";

    #[test]
    fn test_eth_msg_hashing() {
        let msg = verification_msg(TEST_ACCOUNT);
        let eth_hashed_msg = eth_hash_message(&msg);
        assert_eq!(ethers::utils::hash_message(&msg).as_bytes(), eth_hashed_msg);
    }

    #[tokio::test]
    async fn ethereum_ecdsa() {
        // check ethereum specific message hashing
        let msg = verification_msg(TEST_ACCOUNT);
        let eth_hashed_msg = eth_hash_message(&msg);

        // generate signers
        let seed = [2u8; 32];
        let sp_signer = sp_core::ecdsa::Pair::from_seed_slice(&seed).unwrap();
        let signing_key = SigningKey::from_bytes(&seed).unwrap();
        let eth_signer = LocalWallet::from(signing_key);

        // generate signatures
        let mut eth_signature = eth_signer.sign_message(&msg).await.unwrap().to_vec();
        eth_signature[64] -= 27; // 'v' is normalized via eip-155
        let sp_signature = sp_signer.sign_prehashed(&eth_hashed_msg);
        assert_eq!(eth_signature, sp_signature.0);

        // recover encoded key
        let recovered_key = sp_signature.recover_prehashed(&eth_hashed_msg).unwrap();
        let eth_pk = PublicKey::from(eth_signer.signer().verifying_key()).to_encoded_point(true); // encode = true
        assert_eq!(&recovered_key.0, eth_pk.as_bytes());

        // check a signature generated via ethers
        let sp_signature = EcdsaSignature(eth_signature.try_into().unwrap());
        let sp_address = Identity::Address20(eth_signer.address().to_fixed_bytes());
        let id_with_auth = IdentityWithAuth::Ecdsa(sp_address, sp_signature);

        assert!(id_with_auth.verify(&msg));
        assert!(!id_with_auth.verify(b"wrong msg"));

        let (address, signature) = test_ecdsa_id_with_auth(seed, &msg);
        assert!(IdentityWithAuth::Ecdsa(address, signature).verify(&msg));
    }

    #[tokio::test]
    async fn prehashed_ecdsa() {
        // signer
        let seed = [2u8; 32];
        let signer = sp_core::ecdsa::Pair::from_seed_slice(&seed).unwrap();

        // prehashed msg
        let msg = verification_msg(TEST_ACCOUNT);
        let mut prehashed_msg = [0u8; 32];
        let mut hasher = sha3::Sha3_256::new();
        hasher.update(&msg);
        hasher.finalize_into((&mut prehashed_msg).into());

        // signature
        let signature = EcdsaSignature(signer.sign_prehashed(&prehashed_msg).0);
        let id = Identity::Address32(signer.public().0[1..].try_into().unwrap());
        let id_with_auth = IdentityWithAuth::Ecdsa(id, signature);

        assert!(id_with_auth.verify(prehashed_msg));
        // doesn't work with the original msg
        assert!(!id_with_auth.verify(msg));
    }

    #[test]
    fn generic_edwards() {
        let msg = verification_msg(TEST_ACCOUNT);
        let seed = [2u8; 32];
        let signer = sp_core::ed25519::Pair::from_seed_slice(&seed).unwrap();

        let signature = Ed25519Signature(signer.sign(msg.as_ref()).0);
        let address = Identity::Address32(signer.public().0);
        let id_with_auth = IdentityWithAuth::Ed25519(address, signature);

        assert!(id_with_auth.verify(&msg));
        assert!(!id_with_auth.verify(b"wrong msg"));
    }

    #[test]
    fn generic_ristretto() {
        let msg = verification_msg(TEST_ACCOUNT);
        let seed = [2u8; 32];
        let signer = sp_core::sr25519::Pair::from_seed_slice(&seed).unwrap();

        let signature = Sr25519Signature(signer.sign(msg.as_ref()).0);
        let address = Identity::Address32(signer.public().0);
        let id_with_auth = IdentityWithAuth::Sr25519(address, signature);

        assert!(id_with_auth.verify(&msg));
        assert!(!id_with_auth.verify(b"wrong msg"));
    }

    #[test]
    fn other_identities() {
        let id_with_auth = IdentityWithAuth::Other(Identity::Other([0u8; 64]), [0u8; 64]);
        assert!(id_with_auth.verify(b""));
        let id_with_auth = IdentityWithAuth::Other(Identity::Address20([0u8; 20]), [0u8; 64]);
        assert!(!id_with_auth.verify(b""));
        let id_with_auth = IdentityWithAuth::Other(Identity::Address32([0u8; 32]), [0u8; 64]);
        assert!(!id_with_auth.verify(b""));
    }

    #[test]
    fn invalid_crypto_signatures() {
        let address = Identity::Address20([0u8; 20]);
        let signature = Ed25519Signature([0u8; 64]);
        let id_with_auth = IdentityWithAuth::Ed25519(address, signature);
        assert!(!id_with_auth.verify(""));

        let address = Identity::Address20([0u8; 20]);
        let signature = Sr25519Signature([0u8; 64]);
        let id_with_auth = IdentityWithAuth::Sr25519(address, signature);
        assert!(!id_with_auth.verify(""));

        let address = Identity::Address32([0u8; 32]);
        let signature = EcdsaSignature([0u8; 65]);
        let id_with_auth = IdentityWithAuth::Ecdsa(address, signature);
        assert!(!id_with_auth.verify(""));
    }

    #[tokio::test]
    async fn parse() {
        let seed = [2u8; 32];
        let sp_signer = sp_core::ecdsa::Pair::from_seed_slice(&seed).unwrap();
        let signing_key = SigningKey::from_bytes(&seed).unwrap();
        let eth_signer = LocalWallet::from(signing_key);

        let address_bytes = eth_signer.address().to_fixed_bytes();
        let identity = Identity::Address20(address_bytes);

        let msg = "hello";
        let prehashed = eth_hash_message(msg);
        let sp_signature = sp_signer.sign_prehashed(&prehashed);
        let eth_signature = eth_signer.sign_message(msg).await.unwrap().to_vec();
        assert_eq!(&sp_signature.0[0..64], &eth_signature[0..64]);
        assert_ne!(&sp_signature.0[64], &eth_signature[64]);

        let sp_sig_string = hex::encode(sp_signature.0);
        let eth_sig_string = hex::encode(&eth_signature);
        let address_string = hex::encode(eth_signer.address().to_fixed_bytes());

        let sp_id_with_auth = IdentityWithAuth::from_evm(&address_string, &sp_sig_string);
        let eth_id_with_auth = IdentityWithAuth::from_evm(&address_string, &eth_sig_string);

        assert_eq!(
            sp_id_with_auth,
            Ok(IdentityWithAuth::Ecdsa(
                identity,
                EcdsaSignature(sp_signature.into())
            ))
        );

        assert_eq!(sp_id_with_auth, eth_id_with_auth);

        assert_eq!(
            IdentityWithAuth::from_evm("hello", &eth_sig_string),
            Err("invalid address")
        );
        assert_eq!(
            IdentityWithAuth::from_evm(&address_string, "hello"),
            Err("invalid signature")
        );
    }
}
