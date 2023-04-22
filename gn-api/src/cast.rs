#![allow(unused)]
use crate::{runtime, AccountId};
use gn_common::filter::{Filter, Logic as FilterLogic};
use gn_common::merkle::Proof as MerkleProof;
use gn_common::Role;
use paste::paste;

type Guild = gn_common::Guild<AccountId>;
type GuildFilter = gn_common::filter::Guild;
type RuntimeFilter = runtime::runtime_types::gn_common::filter::Filter;
type RuntimeFilterLogic = runtime::runtime_types::gn_common::filter::Logic;
type RuntimeGuild = runtime::runtime_types::gn_common::Guild<AccountId>;
type RuntimeGuildFilter = runtime::runtime_types::gn_common::filter::Guild;
type RuntimeMerkleProof = runtime::runtime_types::gn_common::merkle::Proof;
type RuntimeRole = runtime::runtime_types::gn_common::Role;

macro_rules! cast {
    ($a:ident, $name:tt) => {
        paste! {
            pub mod $name {
                use super::*;
                pub fn to_runtime(input: $a) -> [<Runtime $a>] {
                    unsafe { std::mem::transmute::<$a, [<Runtime $a>]>(input) }
                }
                pub fn from_runtime(input: [<Runtime $a>]) -> $a {
                    unsafe { std::mem::transmute::<[<Runtime $a>], $a>(input) }
                }
            }
        }
    };
}

cast!(Filter, filter);
cast!(FilterLogic, filter_logic);
cast!(Guild, guild);
cast!(GuildFilter, guild_filter);
cast!(Role, role);
cast!(MerkleProof, proof);
