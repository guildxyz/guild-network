mod auth;
pub use auth::*;

use crate::{Decode, Encode, TypeInfo};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Encode, Decode, TypeInfo, Eq, PartialEq, Clone, Copy, Debug)]
pub enum Identity {
    Address20([u8; 20]),
    Address32([u8; 32]),
    Other([u8; 64]),
}

impl AsRef<[u8]> for Identity {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Address20(x) => x,
            Self::Address32(x) => x,
            Self::Other(x) => x,
        }
    }
}

impl From<IdentityWithAuth> for Identity {
    fn from(id_with_auth: IdentityWithAuth) -> Self {
        match id_with_auth {
            IdentityWithAuth::Ecdsa(id, _) => id,
            IdentityWithAuth::Ed25519(id, _) => id,
            IdentityWithAuth::Sr25519(id, _) => id,
            IdentityWithAuth::Other(id, _) => id,
        }
    }
}

impl From<&IdentityWithAuth> for Identity {
    fn from(id_with_auth: &IdentityWithAuth) -> Self {
        match id_with_auth {
            IdentityWithAuth::Ecdsa(id, _) => *id,
            IdentityWithAuth::Ed25519(id, _) => *id,
            IdentityWithAuth::Sr25519(id, _) => *id,
            IdentityWithAuth::Other(id, _) => *id,
        }
    }
}

impl Serialize for Identity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Identity::Address20(b) => serializer.serialize_bytes(b),
            Identity::Address32(b) => serializer.serialize_bytes(b),
            Identity::Other(b) => serializer.serialize_bytes(b),
        }
    }
}

impl<'de> Deserialize<'de> for Identity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let b = <serde_bytes::ByteBuf>::deserialize(deserializer)?;
        // NOTE unwraps are fine because we check the array's length
        match b.len() {
            20 => Ok(Identity::Address20(b.as_ref().try_into().unwrap())),
            32 => Ok(Identity::Address32(b.as_ref().try_into().unwrap())),
            64 => Ok(Identity::Other(b.as_ref().try_into().unwrap())),
            _ => Err(serde::de::Error::invalid_length(b.len(), &"20, 32 or 64")),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_test::{assert_tokens, Token};

    macro_rules! check_conversions {
        ($id:expr) => {
            let id_with_auth = IdentityWithAuth::Ecdsa($id, EcdsaSignature([0u8; 65]));
            assert_eq!($id, (&id_with_auth).into());
            assert_eq!($id, id_with_auth.into());
            let id_with_auth = IdentityWithAuth::Ed25519($id, Ed25519Signature([0u8; 64]));
            assert_eq!($id, (&id_with_auth).into());
            assert_eq!($id, id_with_auth.into());
            let id_with_auth = IdentityWithAuth::Sr25519($id, Sr25519Signature([0u8; 64]));
            assert_eq!($id, (&id_with_auth).into());
            assert_eq!($id, id_with_auth.into());
            let id_with_auth = IdentityWithAuth::Other($id, [0u8; 64]);
            assert_eq!($id, (&id_with_auth).into());
            assert_eq!($id, id_with_auth.into());
        };
    }

    macro_rules! check_serde {
        ($variant:ident, $len: expr) => {
            let id = Identity::$variant([0u8; $len]);
            let expected = Token::BorrowedBytes(&[0u8; $len]);
            assert_tokens(&id, &[expected]);

            let serialized = serde_json::to_vec(&id).unwrap();
            assert_eq!(serde_json::from_slice::<Identity>(&serialized).unwrap(), id);
        };
    }

    #[test]
    fn conversions() {
        check_conversions!(Identity::Address20([0u8; 20]));
        check_conversions!(Identity::Address32([0u8; 32]));
        check_conversions!(Identity::Other([0u8; 64]));
    }

    #[test]
    fn serde_roundtrip() {
        check_serde!(Address20, 20);
        check_serde!(Address32, 32);
        check_serde!(Other, 64);
    }
}
