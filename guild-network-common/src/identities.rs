use ethereum_types::{Address, Signature as EvmSignature};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Identity {
    EvmChain(Address),
    Discord(Vec<u8>),
    Telegram(Vec<u8>),
}

#[derive(Serialize, Deserialize)]
pub enum IdentityAuth {
    EvmChain {
        signature: EvmSignature,
        msg: Vec<u8>,
    },
    Discord,  // not authenticating for now
    Telegram, // not authenticating for now
}
