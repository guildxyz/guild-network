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
    Allowlist(Root, Logic),
    Guild(Guild, Logic),
}

pub fn allowlist_filter<H, A>(allowlist: A, logic: Logic) -> Filter<H::Out>
where
    A: IntoIterator,
    A::Item: AsRef<[u8]>,
    H: hash_db::Hasher,
    H::Out: PartialOrd,
{
    let root = merkle::merkle_root::<H, _>(allowlist);
    Filter::Allowlist(root, logic)
}
