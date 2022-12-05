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
