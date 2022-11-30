use ethereum_types::{Address, U256};
use serde::{Deserialize, Serialize};

pub mod allowlist;
pub mod balance;
pub mod chains;
#[cfg(feature = "with-checks")]
mod check;

use allowlist::Allowlist;
use balance::RequiredBalance;
use chains::EvmChain;

// NOTE example stuff to be implemented
// SolanaBalance(RequiredBalance<Pubkey, u64, SolChain>),
// NearBalance(RequiredBalance<NearAddress, u128, NearChain>),
// SolanaAllowlist(Allowlist<Pubkey>),
// NearAllowlist(Allowlist<NearAddress>),

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Requirement {
    Free,
    EvmBalance(RequiredBalance<Address, U256, EvmChain>),
    EvmAllowlist(Allowlist<Address>),
}
