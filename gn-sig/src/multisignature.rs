use crate::MultiSigner;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::{
    crypto::{AccountId32, ByteArray},
    ecdsa, ed25519, sr25519, RuntimeDebug,
};
use sp_runtime::traits::{Lazy, Verify};

/// Signature verify that can work with any known signature types.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Encode, Decode, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub enum MultiSignature {
    /// An Ed25519 signature.
    Ed25519(ed25519::Signature),
    /// An Sr25519 signature.
    Sr25519(sr25519::Signature),
    /// An ECDSA/SECP256k1 signature.
    Ecdsa(ecdsa::Signature),
}

impl From<ed25519::Signature> for MultiSignature {
    fn from(x: ed25519::Signature) -> Self {
        Self::Ed25519(x)
    }
}

impl TryFrom<MultiSignature> for ed25519::Signature {
    type Error = ();
    fn try_from(m: MultiSignature) -> Result<Self, Self::Error> {
        if let MultiSignature::Ed25519(x) = m {
            Ok(x)
        } else {
            Err(())
        }
    }
}

impl From<sr25519::Signature> for MultiSignature {
    fn from(x: sr25519::Signature) -> Self {
        Self::Sr25519(x)
    }
}

impl TryFrom<MultiSignature> for sr25519::Signature {
    type Error = ();
    fn try_from(m: MultiSignature) -> Result<Self, Self::Error> {
        if let MultiSignature::Sr25519(x) = m {
            Ok(x)
        } else {
            Err(())
        }
    }
}

impl From<ecdsa::Signature> for MultiSignature {
    fn from(x: ecdsa::Signature) -> Self {
        Self::Ecdsa(x)
    }
}

impl TryFrom<MultiSignature> for ecdsa::Signature {
    type Error = ();
    fn try_from(m: MultiSignature) -> Result<Self, Self::Error> {
        if let MultiSignature::Ecdsa(x) = m {
            Ok(x)
        } else {
            Err(())
        }
    }
}

impl Verify for MultiSignature {
    type Signer = MultiSigner;
    fn verify<L: Lazy<[u8]>>(&self, mut msg: L, signer: &AccountId32) -> bool {
        match (self, signer) {
            (Self::Ed25519(ref sig), who) => match ed25519::Public::from_slice(who.as_ref()) {
                Ok(signer) => sig.verify(msg, &signer),
                Err(()) => false,
            },
            (Self::Sr25519(ref sig), who) => match sr25519::Public::from_slice(who.as_ref()) {
                Ok(signer) => sig.verify(msg, &signer),
                Err(()) => false,
            },
            (Self::Ecdsa(ref sig), who) => {
                let prehashed = crate::eth::hash_message(msg.get());
                let Some(pubkey) = crate::eth::recover_prehashed(&prehashed, sig.as_ref()) else {
                    return false
                };

                &crate::eth::pubkey2account(&pubkey) == <dyn AsRef<[u8; 32]>>::as_ref(who)
            }
        }
    }
}
