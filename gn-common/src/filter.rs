use crate::hash::{Hash, Keccak256};
use crate::{GuildName, RoleName, SpVec};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;

#[derive(Encode, Decode, TypeInfo, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Guild {
    pub name: GuildName,
    pub role: Option<RoleName>,
}

#[derive(Encode, Decode, TypeInfo, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Logic {
    And,
    Or,
}

#[derive(Encode, Decode, TypeInfo, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Filter {
    Allowlist(Hash, Logic, u32),
    Guild(Guild, Logic),
}

impl Filter {
    pub fn allowlist<T: Encode>(allowlist: &[T], logic: Logic) -> Self {
        let length = allowlist.len();
        let allowlist: SpVec<SpVec<u8>> = allowlist.iter().map(|item| item.encode()).collect();
        let root = crate::merkle::root::<Keccak256, _>(allowlist);
        Filter::Allowlist(root, logic, length as u32)
    }
}
