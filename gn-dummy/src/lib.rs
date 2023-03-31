#![cfg_attr(not(feature = "std"), no_std)]
#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(unused_crate_dependencies)]

use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::{
    crypto::{self, AccountId32, ByteArray},
    ecdsa, ed25519, sr25519, RuntimeDebug, H160, H256,
};
use sp_runtime::traits::{IdentifyAccount, Lazy, Verify};

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

/// Public key for any known crypto algorithm.
#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum MultiSigner {
    /// An Ed25519 identity.
    Ed25519(ed25519::Public),
    /// An Sr25519 identity.
    Sr25519(sr25519::Public),
    /// An SECP256k1/ECDSA identity
    Ecdsa(H160),
}

/// NOTE: This implementations is required by `SimpleAddressDeterminer`,
/// we convert the hash into some AccountId, it's fine to use any scheme.
impl<T: Into<H256>> crypto::UncheckedFrom<T> for MultiSigner {
    fn unchecked_from(x: T) -> Self {
        ed25519::Public::unchecked_from(x.into()).into()
    }
}

impl AsRef<[u8]> for MultiSigner {
    fn as_ref(&self) -> &[u8] {
        match *self {
            Self::Ed25519(ref who) => who.as_ref(),
            Self::Sr25519(ref who) => who.as_ref(),
            Self::Ecdsa(ref who) => who.as_ref(),
        }
    }
}

impl IdentifyAccount for MultiSigner {
    type AccountId = AccountId32;
    fn into_account(self) -> AccountId32 {
        match self {
            Self::Ed25519(who) => <[u8; 32]>::from(who).into(),
            Self::Sr25519(who) => <[u8; 32]>::from(who).into(),
            Self::Ecdsa(who) => sp_io::hashing::blake2_256(who.as_ref()).into(),
        }
    }
}

impl From<ed25519::Public> for MultiSigner {
    fn from(x: ed25519::Public) -> Self {
        Self::Ed25519(x)
    }
}

impl TryFrom<MultiSigner> for ed25519::Public {
    type Error = ();
    fn try_from(m: MultiSigner) -> Result<Self, Self::Error> {
        if let MultiSigner::Ed25519(x) = m {
            Ok(x)
        } else {
            Err(())
        }
    }
}

impl From<sr25519::Public> for MultiSigner {
    fn from(x: sr25519::Public) -> Self {
        Self::Sr25519(x)
    }
}

impl TryFrom<MultiSigner> for sr25519::Public {
    type Error = ();
    fn try_from(m: MultiSigner) -> Result<Self, Self::Error> {
        if let MultiSigner::Sr25519(x) = m {
            Ok(x)
        } else {
            Err(())
        }
    }
}

impl From<H160> for MultiSigner {
    fn from(x: H160) -> Self {
        Self::Ecdsa(x)
    }
}

impl TryFrom<MultiSigner> for H160 {
    type Error = ();
    fn try_from(m: MultiSigner) -> Result<Self, Self::Error> {
        if let MultiSigner::Ecdsa(x) = m {
            Ok(x)
        } else {
            Err(())
        }
    }
}

#[cfg(feature = "std")]
impl std::fmt::Display for MultiSigner {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::Ed25519(ref who) => write!(fmt, "ed25519: {}", who),
            Self::Sr25519(ref who) => write!(fmt, "sr25519: {}", who),
            Self::Ecdsa(ref who) => write!(fmt, "ecdsa: {}", who),
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
                let mut prefixed_msg = gn_common::identity::ETHEREUM_HASH_PREFIX
                    .as_bytes()
                    .to_vec();
                prefixed_msg.extend_from_slice(msg.get());
                let m = sp_io::hashing::keccak_256(&prefixed_msg);
                match sp_io::crypto::secp256k1_ecdsa_recover_compressed(sig.as_ref(), &m) {
                    Ok(pubkey) => {
                        &sp_io::hashing::keccak_256(pubkey.as_ref())[12..]
                            == <dyn AsRef<[u8; 32]>>::as_ref(who)
                    }
                    _ => false,
                }
            }
        }
    }
}
