use crate::{Decode, Encode, TypeInfo};
use ed25519_zebra::{Signature as EdSig, VerificationKey as EdKey};
use schnorrkel::{PublicKey as SrKey, Signature as SrSig};
use sp_core::keccak_256;

pub const SIGNING_CTX: &[u8] = b"guild-network";

pub type MultiAddress<T> = sp_runtime::MultiAddress<T, u32>;

#[derive(Encode, Decode, TypeInfo, Eq, PartialEq, Clone, Debug)]
pub enum IdentityWithAuth<T> {
    Ecdsa(MultiAddress<T>, sp_core::ecdsa::Signature),
    Ed25519(MultiAddress<T>, sp_core::ed25519::Signature),
    Sr25519(MultiAddress<T>, sp_core::sr25519::Signature),
    Raw(MultiAddress<T>, Vec<u8>),
}

impl<T> IdentityWithAuth<T> {
    pub fn verify<M: AsRef<[u8]>>(&self, msg: M) -> bool {
        match self {
            Self::Ecdsa(MultiAddress::Address20(address), sig) => {
                let Some(recovered_pk) = sig.recover(msg) else {
                    return false
                };
                debug_assert_eq!(recovered_pk.0[0], 0x04);
                &keccak_256(&recovered_pk.0[1..])[12..] == address
            }
            Self::Ecdsa(MultiAddress::Address32(pubkey), sig) => {
                let Some(recovered_pk) = sig.recover(msg) else {
                    return false
                };
                debug_assert_eq!(recovered_pk.0[0], 0x04);
                &recovered_pk.0[1..] == pubkey
            }
            Self::Ed25519(MultiAddress::Address32(pubkey), sig) => {
                let Ok(ed_key) = EdKey::try_from(pubkey.as_ref()) else {
                    return false
                };

                let Ok(ed_sig) = EdSig::try_from(&sig.0[..]) else {
                    return false
                };

                ed_key.verify(&ed_sig, msg.as_ref()).is_ok()
            }
            Self::Sr25519(MultiAddress::Address32(pubkey), sig) => {
                let Ok(sr_key) = SrKey::from_bytes(pubkey.as_ref()) else {
                    return false
                };

                let Ok(sr_sig) = SrSig::from_bytes(&sig.0) else {
                    return false
                };

                sr_key
                    .verify_simple(SIGNING_CTX, msg.as_ref(), &sr_sig)
                    .is_ok()
            }
            Self::Raw(_, _) => true, // not authenticating for now
            _ => false,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::verification_msg;
    use ethers::signers::{LocalWallet, Signer as EthSigner};
    use sp_core::ecdsa::Signature as EcdsaSig;

    const TEST_ACCOUNT: &str = "test-account-0xabcde";

    #[tokio::test]
    async fn ecdsa_sig() {
        let mut rng = StdRng::seed_from_u64(0);
        let signer = LocalWallet::new(&mut rng);

        let signature = signer
            .sign_message(verification_msg(TEST_ACCOUNT))
            .await
            .unwrap();
        let address = signer.address();

        let sp_signature = MultiSignature::Ecdsa(EcdsaSig::from_raw(signature.as_bytes()));
        let sp_address = MultiAddress::Address20(address.to_fixed_bytes());
    }
}
