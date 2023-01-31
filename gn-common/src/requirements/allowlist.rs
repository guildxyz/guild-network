use parity_scale_codec::alloc::vec::Vec;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Allowlist<T>(Vec<T>);

impl<T: PartialEq> Allowlist<T> {
    pub fn new(list: Vec<T>) -> Self {
        Self(list)
    }
    pub fn is_member(&self, identifier: &T) -> bool {
        self.0.iter().any(|id| id == identifier)
    }
}

impl<T> From<Vec<T>> for Allowlist<T> {
    fn from(value: Vec<T>) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod test {
    use super::Allowlist;

    #[test]
    fn is_member() {
        let allowlist = Allowlist(codec::alloc::vec![0, 2]);

        assert!(allowlist.is_member(&0));
        assert!(!allowlist.is_member(&1));
        assert!(allowlist.is_member(&2));
    }
}
