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

use parity_scale_codec::alloc::vec::Vec as SpVec;
use serde::{Deserialize, Serialize};
use serde_cbor::{from_slice as cbor_deserialize, to_vec as cbor_serialize};

pub type EvmAddress = [u8; 20];
pub type U256 = [u8; 32];

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Requirement {
    EvmBalance(Balance<EvmAddress, U256, EvmChain>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequirementsWithLogic {
    pub requirements: SpVec<Requirement>,
    pub logic: parity_scale_codec::alloc::string::String,
}

impl RequirementsWithLogic {
    pub fn into_serialized_tuple(
        self,
    ) -> Result<gn_common::SerializedRequirements, serde_cbor::Error> {
        let reqs = self
            .requirements
            .into_iter()
            .map(|x| cbor_serialize(&x))
            .collect::<Result<SpVec<_>, _>>()?;
        let logic = cbor_serialize(&self.logic)?;
        Ok((reqs, logic))
    }

    pub fn from_serialized_tuple(
        tuple: gn_common::SerializedRequirements,
    ) -> Result<Self, serde_cbor::Error> {
        let requirements = tuple
            .0
            .into_iter()
            .map(|x| cbor_deserialize(&x))
            .collect::<Result<SpVec<_>, _>>()?;
        let logic = cbor_deserialize(&tuple.1)?;
        Ok(Self {
            requirements,
            logic,
        })
    }
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
