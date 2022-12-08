use gn_common::{GuildName, RoleName};
use gn_gate::requirements::Requirement;
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
    pub requirements: RequirementsLogic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementsLogic {
    pub logic: String,
    pub requirements: Vec<Requirement>,
}
