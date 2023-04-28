#![allow(unused)]
use crate::{runtime, AccountId, OracleRequest};
use gn_common::filter::{Filter, Logic as FilterLogic};
use gn_common::merkle::Proof as MerkleProof;
use gn_common::{Identity, Prefix, Role};
use paste::paste;

type AddressMap = std::collections::BTreeMap<Prefix, AddressVec>;
type AddressVec = Vec<AccountId>;
type IdentityMap = std::collections::BTreeMap<Prefix, Identity>;
type Guild = gn_common::Guild<AccountId>;
type GuildFilter = gn_common::filter::Guild;
type RuntimeAddressMap =
    runtime::runtime_types::bounded_collections::bounded_btree_map::BoundedBTreeMap<
        Prefix,
        RuntimeAddressVec,
    >;
type RuntimeAddressVec =
    runtime::runtime_types::bounded_collections::bounded_vec::BoundedVec<AccountId>;
type RuntimeFilter = runtime::runtime_types::gn_common::filter::Filter;
type RuntimeFilterLogic = runtime::runtime_types::gn_common::filter::Logic;
type RuntimeGuild = runtime::runtime_types::gn_common::Guild<AccountId>;
type RuntimeGuildFilter = runtime::runtime_types::gn_common::filter::Guild;
type RuntimeIdentityMap =
    runtime::runtime_types::bounded_collections::bounded_btree_map::BoundedBTreeMap<
        Prefix,
        Identity,
    >;
type RuntimeMerkleProof = runtime::runtime_types::gn_common::merkle::Proof;
type RuntimeOracleRequest = runtime::runtime_types::gn_common::OracleRequest<AccountId, u32, u128>;
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
cast!(OracleRequest, oracle_request);
cast!(Role, role);
cast!(MerkleProof, proof);
cast!(IdentityMap, identity_map);
cast!(AddressMap, address_map);
