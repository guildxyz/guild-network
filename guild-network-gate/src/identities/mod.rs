use ethereum_types::{Address, Signature as EvmSignature};
use serde::{Deserialize, Serialize};

#[cfg(feature = "with-checks")]
mod impls;
#[cfg(feature = "with-checks")]
mod map;
#[cfg(feature = "with-checks")]
pub use map::*;

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
