use ethereum_types::{Address, U256};
use serde::{Deserialize, Serialize};

pub mod allowlist;
pub mod balance;
#[cfg(feature = "with-checks")]
mod check;

use allowlist::Allowlist;
use balance::RequiredBalance;

#[derive(Deserialize, Serialize)]
pub enum Requirement {
    Free,
    EthereumBalance(RequiredBalance<Address, U256>),
    BscBalance(RequiredBalance<Address, U256>),
    GnosisBalance(RequiredBalance<Address, U256>),
    PolygonBalance(RequiredBalance<Address, U256>),
    EvmAllowlist(Allowlist<Address>),
}
