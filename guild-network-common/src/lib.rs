#![no_std]

use codec::alloc::vec::Vec;
use codec::{Decode, Encode};

pub type GuildName = [u8; 32];
pub type RoleName = [u8; 32];

pub type OperatorIdentifier = u64;
pub type RequestIdentifier = u64;

#[derive(Encode, Decode, Clone)]
pub struct JoinRequest<T> {
    pub requester: T,
    pub guild_name: GuildName,
    pub role_name: RoleName,
    pub requester_identities: Vec<u8>,
    pub request_data: Vec<u8>,
}

#[derive(Encode, Decode, Clone)]
pub struct JoinRequestWithAccess<T> {
    pub access: bool,
    pub request: JoinRequest<T>,
}
