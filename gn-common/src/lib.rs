#![cfg_attr(not(feature = "std"), no_std)]
#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(unused_crate_dependencies)]

pub mod filter;
pub mod hash;
pub mod merkle;
pub mod pad;
pub mod utils;

use parity_scale_codec::alloc::vec::Vec as SpVec;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;

pub const OFFCHAIN_ALLOWLIST_PREFIX: &[u8] = b"guild-allowlist";

pub fn offchain_allowlist_key(key: &[u8]) -> SpVec<u8> {
    let mut offchain_key = SpVec::from(OFFCHAIN_ALLOWLIST_PREFIX);
    offchain_key.extend_from_slice(key);
    offchain_key
}

pub type GuildName = [u8; 32];
pub type RoleName = [u8; 32];

pub type Authority = [u8; 32];
pub type Identity = [u8; 32];
pub type OperatorIdentifier = u64;
pub type Prefix = [u8; 8];
pub type RequestIdentifier = u64;
pub type SerializedData = SpVec<u8>;
pub type SerializedRequirements = (SpVec<SerializedData>, SerializedData);

#[derive(Encode, Decode, TypeInfo, Clone, Debug, Eq, PartialEq)]
pub struct Guild<T> {
    pub name: GuildName,
    pub owner: T,
    pub metadata: SerializedData,
    pub roles: SpVec<RoleName>,
}

#[derive(Encode, Decode, TypeInfo, Clone, Debug, Eq, PartialEq)]
pub struct Role {
    pub filter: Option<filter::Filter>,
    pub requirements: Option<SerializedRequirements>,
}

#[derive(Encode, Decode, TypeInfo, Clone, Debug, Eq, PartialEq)]
pub struct LinkIdentityRequest<T> {
    pub requester: T,
    pub prefix: Prefix,
    pub identity: Identity,
}

#[derive(Encode, Decode, TypeInfo, Clone, Debug, Eq, PartialEq)]
pub struct AccessCheckRequest<T> {
    pub requester: T,
    pub account: T,
    pub guild_name: GuildName,
    pub role_name: GuildName,
}
