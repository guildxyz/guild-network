#![allow(unused)]
use crate::{runtime, AccountId};
use gn_common::filter::{Filter, Logic as FilterLogic};
use gn_common::identity::{Identity, IdentityWithAuth};
use gn_common::merkle::Proof as MerkleProof;
use gn_common::Role;
use paste::paste;

type IdentityVec = Vec<Identity>;
type Guild = gn_common::Guild<AccountId>;
type GuildFilter = gn_common::filter::Guild;
type RuntimeFilter = runtime::runtime_types::gn_common::filter::Filter;
type RuntimeFilterLogic = runtime::runtime_types::gn_common::filter::Logic;
type RuntimeGuild = runtime::runtime_types::gn_common::Guild<AccountId>;
type RuntimeGuildFilter = runtime::runtime_types::gn_common::filter::Guild;
type RuntimeIdentity = runtime::runtime_types::gn_common::identity::Identity;
type RuntimeIdentityVec = Vec<RuntimeIdentity>;
type RuntimeIdentityWithAuth = runtime::runtime_types::gn_common::identity::auth::IdentityWithAuth;
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
cast!(Identity, id);
cast!(Guild, guild);
cast!(GuildFilter, guild_filter);
cast!(Role, role);
cast!(IdentityVec, id_vec);
cast!(IdentityWithAuth, id_with_auth);
cast!(MerkleProof, proof);
