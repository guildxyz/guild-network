#![cfg_attr(not(feature = "std"), no_std)]
#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(unused_crate_dependencies)]

pub mod balance;
pub mod chains;
#[cfg(feature = "check")]
mod check;

use balance::Balance;
use chains::EvmChain;

use serde::{Deserialize, Serialize};

pub type EvmAddress = [u8; 20];
pub type U256 = [u8; 32];

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Requirement {
    EvmBalance(Balance<EvmAddress, U256, EvmChain>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequirementsWithLogic {
    pub requirements: parity_scale_codec::alloc::vec::Vec<Requirement>,
    pub logic: parity_scale_codec::alloc::string::String,
}

// to avoid unused crate dependencies
//
// these dev-dependencies are used only when the `check` feature is enabled,
// but dev-dependencies cannot be optional
#[cfg(test)]
mod test {
    use async_trait as _;
    use tokio as _;
}
