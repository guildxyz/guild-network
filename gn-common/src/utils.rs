use crate::Encode;
use scale_info::prelude::format;
use scale_info::prelude::string::String as SpString;

pub fn verification_msg<T: Encode>(id: T) -> SpString {
    format!("Guild Network registration id: {:?}", id.encode())
}
