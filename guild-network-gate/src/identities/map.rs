use super::{Identity, IdentityWithAuth};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum IdentityType {
    EvmChain,
    Discord,
    Telegram,
}

pub struct IdentityMap(HashMap<IdentityType, Identity>);

impl IdentityMap {
    pub fn from_verified_identities(
        ids: Vec<IdentityWithAuth>,
        verification_msg: &str,
    ) -> Result<Self, anyhow::Error> {
        let map = ids
            .into_iter()
            .map(|id| {
                id.verify(verification_msg)?;
                Ok::<(IdentityType, Identity), anyhow::Error>((
                    id.as_identity_type(),
                    Identity::from(id),
                ))
            })
            .collect::<Result<_, _>>()?;
        Ok(Self(map))
    }

    pub fn into_identity_vec(self) -> Vec<Identity> {
        self.0.into_values().collect()
    }

    pub fn inner(&self) -> &HashMap<IdentityType, Identity> {
        &self.0
    }
}
