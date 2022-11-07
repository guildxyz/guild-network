pub mod identities;
pub mod requirements;

#[macro_export]
macro_rules! address {
    ($addr:expr) => {{
        <ethereum_types::Address as std::str::FromStr>::from_str($addr)
            .expect(&format!("invalid address {}", $addr))
    }};
}
