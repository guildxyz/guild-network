use ethereum_types::{Address, Signature as EvmSignature};
use serde::{Deserialize, Serialize};

#[cfg(feature = "with-checks")]
mod impls;
#[cfg(feature = "with-checks")]
mod map;
#[cfg(feature = "with-checks")]
pub use map::*;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum Platform {
    EvmChain,
    Discord,
    Telegram,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Identity {
    EvmChain(Address),
    Discord(Vec<u8>),
    Telegram(Vec<u8>),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum IdentityWithAuth {
    EvmChain(Address, EvmSignature),
    Discord(Vec<u8>, ()),  // not authenticating for now
    Telegram(Vec<u8>, ()), // not authenticating for now
}

impl IdentityWithAuth {
    pub fn into_platform_with_id(self) -> (Platform, Identity) {
        match self {
            Self::EvmChain(address, _) => (Platform::EvmChain, Identity::EvmChain(address)),
            Self::Discord(id, _) => (Platform::Discord, Identity::Discord(id)),
            Self::Telegram(id, _) => (Platform::Telegram, Identity::Telegram(id)),
        }
    }
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
