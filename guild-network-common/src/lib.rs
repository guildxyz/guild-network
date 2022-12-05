#![cfg_attr(not(feature = "std"), no_std)]
#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]

pub mod identities;
pub mod pad;
pub mod requirements;
pub mod utils;

pub use codec::alloc::vec::Vec as SpVec;
pub use codec::{Decode, Encode};
pub use scale_info::TypeInfo;

pub type EvmAddress = [u8; 20];
pub type EvmSignature = [u8; 65];
pub type GuildName = [u8; 32];
pub type RoleName = [u8; 32];
pub type U256 = [u8; 32];

pub type OperatorIdentifier = u64;
pub type RequestIdentifier = u64;

#[derive(Encode, Decode, TypeInfo, Clone, Debug)]
pub struct Request<T> {
    pub requester: T,
    pub data: RequestData,
}

#[derive(Encode, Decode, TypeInfo, Clone, Debug)]
pub enum RequestData {
    Register(Vec<identities::IdentityWithAuth>),
    Join { guild: GuildName, role: RoleName },
}
