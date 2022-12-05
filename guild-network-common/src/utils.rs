pub fn matches_variant<T>(a: &T, b: &T) -> bool {
    core::mem::discriminant(a) == core::mem::discriminant(b)
}

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
