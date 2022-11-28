pub mod identities;
pub mod requirements;

pub use ethereum_types::{Address as EvmAddress, Signature as EvmSignature};

#[macro_export]
macro_rules! address {
    ($addr:expr) => {{
        <EvmAddress as std::str::FromStr>::from_str($addr)
            .expect(&format!("invalid address {}", $addr))
    }};
}
