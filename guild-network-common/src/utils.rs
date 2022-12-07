pub fn matches_variant<T>(a: &T, b: &T) -> bool {
    core::mem::discriminant(a) == core::mem::discriminant(b)
}

#[cfg(feature = "std")]
pub fn verification_msg<T: std::fmt::Display>(id: T) -> String {
    format!("This is my ({id}) registration request to Guild Network")
}
