#![cfg_attr(not(feature = "std"), no_std)]
#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]

pub mod identities;
pub mod requirements;

pub type EvmAddress = [u8; 20];
pub type EvmSignature = [u8; 65];
pub type U256 = [u8; 32];

#[cfg(feature = "std")]
pub fn verification_msg<T, U, V>(id: T, guild_name: U, role_name: V) -> String
where
    T: std::fmt::Display,
    U: std::fmt::Display,
    V: std::fmt::Display,
{
    format!(
        "{} wants to join role {} of guild {}",
        id, guild_name, role_name
    )
}
