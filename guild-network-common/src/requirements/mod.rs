pub mod allowlist;
pub mod balance;
#[cfg(feature = "with-checks")]
mod check;

use allowlist::Allowlist;
use balance::RequiredBalance;
use crate::{EvmAddress, U256};
use providers::EvmChain;
use serde::{Deserialize, Serialize};

// NOTE example stuff to be implemented
// SolanaBalance(RequiredBalance<Pubkey, u64, SolChain>),
// NearBalance(RequiredBalance<NearAddress, u128, NearChain>),
// SolanaAllowlist(Allowlist<Pubkey>),
// NearAllowlist(Allowlist<NearAddress>),

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Requirement {
    Free,
    EvmBalance(RequiredBalance<EvmAddress, U256, EvmChain>),
    EvmAllowlist(Allowlist<EvmAddress>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequirementsWithLogic {
    pub logic: codec::alloc::string::String,
    pub requirements: codec::alloc::vec::Vec<Requirement>,
}
