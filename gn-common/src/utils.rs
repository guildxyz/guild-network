use crate::Encode;
use scale_info::prelude::format;
use scale_info::prelude::string::String as SpString;

pub fn verification_msg<T: Encode>(id: T) -> SpString {
    format!(
        "Guild Network registration id: {}",
        hex::encode(id.encode())
    )
}

#[cfg(feature = "std")]
pub fn matches_variant<T>(a: &T, b: &T) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}
