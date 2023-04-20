use crate::hash::{Hash, Keccak256};
use crate::{GuildName, Identity, RoleName};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Encode, Decode, TypeInfo, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Guild {
    pub name: GuildName,
    pub role: Option<RoleName>,
}

#[derive(Serialize, Deserialize, Encode, Decode, TypeInfo, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Logic {
    And,
    Or,
}

#[derive(Serialize, Deserialize, Encode, Decode, TypeInfo, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Filter {
    Allowlist(Hash, Logic, u32),
    Guild(Guild, Logic),
}

impl Filter {
    pub fn allowlist(allowlist: &[Identity], logic: Logic) -> Self {
        let length = allowlist.len();
        let root = crate::merkle::root::<Keccak256, _>(allowlist);
        Filter::Allowlist(root, logic, length as u32)
    }
}
