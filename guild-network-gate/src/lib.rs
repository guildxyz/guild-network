pub mod identities;
pub mod requirements;

pub use ethereum_types::{Address as EvmAddress, Signature as EvmSignature};

#[macro_export]
macro_rules! address {
    ($addr:expr) => {{
        <ethereum_types::Address as std::str::FromStr>::from_str($addr)
            .expect(&format!("invalid address {}", $addr))
    }};
}

pub fn verification_msg<T, U, V>(id: T, guild_name: U, role_name: V) -> String
where
    T: std::fmt::Display,
    U: std::fmt::Display,
    V: std::fmt::Display,
{
    format!("{id} wants to join role {guild_name} of guild {role_name}")
}
