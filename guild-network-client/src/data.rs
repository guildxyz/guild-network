use crate::runtime::runtime_types::pallet_guild::pallet::Guild as RuntimeGuildData;
use crate::AccountId;
use guild_network_common::requirements::RequirementsWithLogic;
use guild_network_common::{GuildName, RoleName};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guild {
    pub name: GuildName,
    pub metadata: Vec<u8>,
    pub roles: Vec<Role>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: RoleName,
    pub reqs: RequirementsWithLogic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildData {
    pub owner: AccountId,
    pub metadata: Vec<u8>,
}

impl From<RuntimeGuildData<AccountId>> for GuildData {
    fn from(value: RuntimeGuildData<AccountId>) -> Self {
        Self {
            owner: value.owner,
            metadata: value.metadata,
        }
    }
}
