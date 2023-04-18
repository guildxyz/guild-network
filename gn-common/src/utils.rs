use crate::Encode;
use scale_info::prelude::format;
use scale_info::prelude::string::String as SpString;

pub fn verification_msg<T: Encode>(id: T) -> SpString {
    format!(
        "Guild Network registration id: {}",
        hex::encode(id.encode())
    )
}

pub fn matches_variant<T>(a: &T, b: &T) -> bool {
    core::mem::discriminant(a) == core::mem::discriminant(b)
}

pub fn evm_to_account(evm_address: [u8; 20]) -> [u8; 32] {
    let mut address = [0u8; 24];
    address[0..4].copy_from_slice(b"evm:");
    address[4..].copy_from_slice(&evm_address);
    crate::hash::blake2256(address)
}
