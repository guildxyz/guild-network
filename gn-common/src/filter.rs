use crate::identity::Identity;
use crate::{GuildName, RoleName};
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
pub enum Filter<Root> {
    Allowlist(Root, Logic, u32),
    Guild(Guild, Logic),
}

pub fn allowlist_filter<H>(allowlist: &[Identity], logic: Logic) -> Filter<H::Out>
where
    H: hash_db::Hasher,
    H::Out: PartialOrd,
{
    let length = allowlist.len();
    let root = merkle::merkle_root::<H, _>(allowlist);
    Filter::Allowlist(root, logic, length as u32)
}
