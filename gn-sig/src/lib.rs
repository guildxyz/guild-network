#![cfg_attr(not(feature = "std"), no_std)]
#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(unused_crate_dependencies)]

mod eth;
mod multisignature;
mod multisigner;
pub mod webcrypto;

pub use multisignature::MultiSignature;
pub use multisigner::MultiSigner;
