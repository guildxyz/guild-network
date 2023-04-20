use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::{
    crypto::{self, AccountId32},
    ed25519, sr25519, RuntimeDebug, H160, H256,
};
use sp_runtime::traits::IdentifyAccount;

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
            Self::Ecdsa(who) => crate::eth::address2account(who.into()).into(),
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
