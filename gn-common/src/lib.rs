#![no_std]
#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]

#[cfg(feature = "pad")]
mod pad;
#[cfg(feature = "pad")]
pub use pad::{pad_to_32_bytes, unpad_from_32_bytes};

use codec::alloc::vec::Vec;
use codec::{Decode, Encode};

pub type GuildName = [u8; 32];
pub type RoleName = [u8; 32];

pub type OperatorIdentifier = u64;
pub type RequestIdentifier = u64;

#[derive(Encode, Decode, Clone)]
pub struct JoinRequest<T> {
    pub requester: T,
    pub requester_identities: Vec<u8>,
    pub guild_name: GuildName,
    pub role_name: RoleName,
}
