#[cfg(feature = "with-checks")]
mod impls;
#[cfg(feature = "with-checks")]
mod map;

#[cfg(feature = "with-checks")]
pub use map::IdentityMap;

use crate::{EvmAddress, EvmSignature};
use codec::{Decode, Encode};

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum Platform {
    EvmChain,
    Discord,
    Telegram,
}

#[derive(Encode, Decode, Clone, Copy, Debug)]
pub enum Identity {
    EvmChain(EvmAddress),
    Discord(u64),
    Telegram(u64),
}

#[derive(Encode, Decode, Clone, Copy, Debug)]
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
