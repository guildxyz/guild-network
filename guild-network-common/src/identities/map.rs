use super::{Identity, IdentityWithAuth, Platform};
use crate::EvmAddress;
use std::collections::HashMap;

/// Stores the user's identities in a HashMap that allows
/// `O(1)` access to platform-specific identities.
pub struct IdentityMap(HashMap<Platform, Identity>);

impl IdentityMap {
    pub fn from_verified_identities(
        ids: Vec<IdentityWithAuth>,
        verification_msg: &str,
    ) -> Result<Self, anyhow::Error> {
        let map = ids
            .into_iter()
            .map(|id| {
                id.verify(verification_msg)?;
                Ok::<(Platform, Identity), anyhow::Error>(id.into_platform_with_id())
            })
            .collect::<Result<_, _>>()?;
        Ok(Self(map))
    }

    pub fn from_identities(ids: Vec<Identity>) -> Self {
        let map = ids.into_iter().map(|x| {
            match x {
                Identity::EvmChain(address) => (Platform::EvmChain, Identity::EvmChain(address)),
                Identity::Discord(id) => (Platform::Discord, Identity::Discord(id)),
                Identity::Telegram(id) => (Platform::Telegram, Identity::Telegram(id)),
            }
        }).collect();

        Self(map)
    }

    pub fn into_identity_vec(self) -> Vec<Identity> {
        self.0.into_values().collect()
    }

    pub fn inner(&self) -> &HashMap<Platform, Identity> {
        &self.0
    }

    pub fn evm_address(&self) -> Option<&EvmAddress> {
        match self.0.get(&Platform::EvmChain) {
            Some(Identity::EvmChain(address)) => Some(address),
            None => None,
            // "identities cannot be messed up when building the map using
            // "from_verified_identities"
            _ => unreachable!(),
        }
    }
}
