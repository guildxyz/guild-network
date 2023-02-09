pub mod allowlist;
pub mod balance;
pub mod chains;
#[cfg(feature = "reqcheck")]
mod check;

use allowlist::Allowlist;
use balance::RequiredBalance;
use chains::EvmChain;

use serde::{Deserialize, Serialize};

pub type EvmAddress = [u8; 20];
pub type U256 = [u8; 32];

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Requirement {
    Free,
    EvmBalance(RequiredBalance<EvmAddress, U256, EvmChain>),
    EvmAllowlist(Allowlist<EvmAddress>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequirementsWithLogic {
    pub logic: parity_scale_codec::alloc::string::String,
    pub requirements: parity_scale_codec::alloc::vec::Vec<Requirement>,
}
