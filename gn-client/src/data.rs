use crate::runtime::runtime_types::pallet_guild::pallet::Guild as RuntimeGuildData;
use crate::AccountId;
use gn_common::pad::unpad_from_32_bytes;
use gn_common::requirements::RequirementsWithLogic;
use gn_common::{GuildName, RoleName};
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
    pub name: String,
    pub owner: AccountId,
    pub metadata: Vec<u8>,
    pub roles: Vec<String>,
}

impl From<RuntimeGuildData<AccountId>> for GuildData {
    fn from(value: RuntimeGuildData<AccountId>) -> Self {
        Self {
            name: unpad_from_32_bytes(&value.name),
            owner: value.data.owner,
            metadata: value.data.metadata,
            roles: value
                .data
                .roles
                .into_iter()
                .map(|role| unpad_from_32_bytes(&role))
                .collect(),
        }
    }
}
