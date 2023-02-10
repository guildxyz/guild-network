use scale_info::prelude::fmt::Debug;
use scale_info::prelude::format;
use scale_info::prelude::string::String as SpString;

pub fn verification_msg<T: Debug>(id: T) -> SpString {
    format!("This is my ({id:?}) registration request to Guild Network")
}
