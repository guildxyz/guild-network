pub fn verification_msg<T: std::fmt::Display>(id: T) -> String {
    format!("This is my ({id}) registration request to Guild Network")
}
