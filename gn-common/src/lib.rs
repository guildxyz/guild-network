#![cfg_attr(not(feature = "std"), no_std)]
#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(unused_crate_dependencies)]

pub mod filter;
pub mod identity;
pub mod pad;
pub mod utils;

pub use merkle::merkle_proof as generate_merkle_proof;
pub use merkle::verify_proof as verify_merkle_proof;
pub use merkle::Leaf as MerkleLeaf;
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

pub type OperatorIdentifier = u64;
pub type RequestIdentifier = u64;
pub type SerializedData = SpVec<u8>;
pub type SerializedRequirements = (SpVec<SerializedData>, SerializedData);

#[derive(Encode, Decode, TypeInfo, Clone, Debug)]
pub struct Request<T> {
    pub requester: T,
    pub data: RequestData<T>,
}

#[derive(Encode, Decode, TypeInfo, Clone, Debug, Eq, PartialEq)]
pub enum RequestData<T> {
    Register {
        identity_with_auth: identity::IdentityWithAuth,
        index: u8,
    },
    ReqCheck {
        account: T,
        guild_name: GuildName,
        role_name: RoleName,
    },
}

#[derive(Encode, Decode, TypeInfo, Clone, Debug, Eq, PartialEq)]
pub struct Guild<T> {
    pub name: GuildName,
    pub owner: T,
    pub metadata: SerializedData,
    pub roles: SpVec<RoleName>,
}

#[derive(Encode, Decode, TypeInfo, Clone, Debug, Eq, PartialEq)]
pub struct Role<H> {
    pub filter: Option<filter::Filter<H>>,
    pub requirements: Option<SerializedRequirements>,
}

#[derive(Encode, Decode, TypeInfo, Clone, Debug, Eq, PartialEq)]
pub struct MerkleProof<T> {
    pub path: SpVec<T>,
    pub id_index: u8,
}
