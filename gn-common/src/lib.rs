#![cfg_attr(not(feature = "std"), no_std)]
#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(unused_crate_dependencies)]

pub mod identity;
pub mod pad;
#[cfg(feature = "std")]
pub mod requirements;
pub mod utils;

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;

pub type GuildName = [u8; 32];
pub type RoleName = [u8; 32];

pub type OperatorIdentifier = u64;
pub type RequestIdentifier = u64;

#[derive(Encode, Decode, TypeInfo, Clone, Debug)]
pub struct Request<T> {
    pub requester: T,
    pub data: RequestData<T>,
}

#[derive(Encode, Decode, TypeInfo, Eq, PartialEq, Clone, Debug)]
pub enum RequestData<T> {
    Register {
        identity_with_auth: identity::IdentityWithAuth,
        index: u8,
    },
    ReqCheck {
        account: T,
        guild: GuildName,
        role: RoleName,
    },
}
