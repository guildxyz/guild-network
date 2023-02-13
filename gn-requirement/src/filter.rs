use gn_common::identity::Identity;
use gn_common::{GuildName, RoleName};
use hash_db::Hasher;
use parity_scale_codec::alloc::vec::Vec as SpVec;

#[derive(Clone, Copy, Debug)]
pub struct Guild {
    pub name: GuildName,
    pub role: Option<RoleName>,
}

#[derive(Clone, Copy, Debug)]
pub enum FilterLogic {
    And,
    Or,
}

pub enum Filter {
    Allowlist(SpVec<Identity>, FilterLogic),
    Guild(Guild, FilterLogic),
}

pub enum OnChainFilter<Root> {
    Allowlist(Root, FilterLogic),
    Guild(Guild, FilterLogic),
}

impl Filter {
    pub fn to_onchain<H>(&self) -> OnChainFilter<H::Out>
    where
        H: Hasher,
        H::Out: PartialOrd,
    {
        match self {
            Self::Allowlist(list, logic) => {
                let root = merkle::merkle_root::<H, _>(list);
                OnChainFilter::Allowlist(root, *logic)
            }
            Self::Guild(guild, logic) => OnChainFilter::Guild(*guild, *logic),
        }
    }
}
