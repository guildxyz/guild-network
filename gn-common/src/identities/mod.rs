#[cfg(feature = "with-checks")]
mod impls;
#[cfg(feature = "with-checks")]
mod map;

#[cfg(feature = "with-checks")]
pub use map::IdentityMap;

use crate::{Decode, Encode, TypeInfo};
use crate::{EvmAddress, EvmSignature};

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum Platform {
    EvmChain,
    Discord,
    Telegram,
}

impl From<Platform> for u32 {
    fn from(value: Platform) -> Self {
        value as u32
    }
}

impl From<&IdentityWithAuth> for Platform {
    fn from(value: &IdentityWithAuth) -> Self {
        match value {
            IdentityWithAuth::EvmChain(_, _) => Self::EvmChain,
            IdentityWithAuth::Discord(_, _) => Self::Discord,
            IdentityWithAuth::Telegram(_, _) => Self::Telegram,
        }
    }
}

impl From<&Identity> for Platform {
    fn from(value: &Identity) -> Self {
        match value {
            Identity::EvmChain(_) => Self::EvmChain,
            Identity::Discord(_) => Self::Discord,
            Identity::Telegram(_) => Self::Telegram,
        }
    }
}

#[derive(Encode, Decode, TypeInfo, Eq, PartialEq, Clone, Copy, Debug)]
pub enum Identity {
    EvmChain(EvmAddress),
    Discord(u64),
    Telegram(u64),
}

#[derive(Encode, Decode, TypeInfo, Eq, PartialEq, Clone, Copy, Debug)]
pub enum IdentityWithAuth {
    EvmChain(EvmAddress, EvmSignature),
    Discord(u64, ()),  // not authenticating for now
    Telegram(u64, ()), // not authenticating for now
}

impl From<IdentityWithAuth> for Identity {
    fn from(value: IdentityWithAuth) -> Self {
        match value {
            IdentityWithAuth::EvmChain(address, _) => Self::EvmChain(address),
            IdentityWithAuth::Discord(id, _) => Self::Discord(id),
            IdentityWithAuth::Telegram(id, _) => Self::Telegram(id),
        }
    }
}