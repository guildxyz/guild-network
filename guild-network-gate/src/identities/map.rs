use super::{Identity, IdentityWithAuth};

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum IdentityType {
    EvmChain,
    Discord,
    Telegram,
}

pub struct IdentityMap(std::collections::HashMap<IdentityType, Identity>);

impl IdentityMap {
    pub fn from_verified_identities(
        ids: Vec<IdentityWithAuth>,
        maybe_msg: Option<&str>,
    ) -> Result<Self, anyhow::Error> {
        let map = ids
            .into_iter()
            .map(|id| {
                id.verify(maybe_msg.as_ref().copied())?;
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
}
