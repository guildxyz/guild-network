use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Allowlist<T>(Vec<T>);

impl<T: PartialEq> Allowlist<T> {
    pub fn is_member(&self, identifier: &T) -> bool {
        self.0.iter().any(|id| id == identifier)
    }
}

#[cfg(test)]
mod test {
    use super::Allowlist;
    use crate::address;

    #[test]
    fn is_member() {
        let allowlist = Allowlist(vec![
            address!("0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE"),
            address!("0x20CC54c7ebc5f43b74866D839b4BD5c01BB23503"),
        ]);

        assert!(allowlist.is_member(&address!("0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE")));
        assert!(allowlist.is_member(&address!("0x20CC54c7ebc5f43b74866D839b4BD5c01BB23503")));
        assert!(!allowlist.is_member(&address!("0x0000000000000000000000000000000000000000")));
    }
}
