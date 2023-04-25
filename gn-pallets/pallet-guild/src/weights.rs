
//! Autogenerated weights for `pallet_guild`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 19.0.0
//! DATE: 2023-02-22, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `razorblade`, CPU: `Intel(R) Core(TM) i5-10210U CPU @ 1.60GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/gn-node
// benchmark
// pallet
// --chain
// dev
// --pallet
// pallet_guild
// --extrinsic
// *
// --execution=wasm
// --wasm-execution=compiled
// --steps
// 50
// --repeat
// 20
// --output
// ./gn-pallets/pallet-guild/src/weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

pub trait WeightInfo {
	fn create_guild() -> Weight;
	fn create_free_role() -> Weight;
	fn create_role_with_allowlist() -> Weight;
	fn create_child_role() -> Weight;
	fn create_unfiltered_role() -> Weight;
	fn join_free_role() -> Weight;
	fn join_child_role() -> Weight;
	fn join_role_with_allowlist() -> Weight;
	fn join_unfiltered_role() -> Weight;
	fn leave() -> Weight;
	fn request_access_check() -> Weight;
}

/// Weight functions for `pallet_guild`.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn create_guild() -> Weight {
		Weight::from_ref_time(1000)
	}
	fn create_free_role() -> Weight {
		Weight::from_ref_time(1000)
	}
	fn create_role_with_allowlist() -> Weight {
		Weight::from_ref_time(1000)
	}
	fn create_child_role() -> Weight {
		Weight::from_ref_time(1000)
	}
	fn create_unfiltered_role() -> Weight {
		Weight::from_ref_time(1000)
	}
	fn join_free_role() -> Weight {
		Weight::from_ref_time(1000)
	}
	fn join_child_role() -> Weight {
		Weight::from_ref_time(1000)
	}
	fn join_role_with_allowlist() -> Weight {
		Weight::from_ref_time(1000)
	}
	fn join_unfiltered_role() -> Weight {
		Weight::from_ref_time(1000)
	}
	fn leave() -> Weight {
		Weight::from_ref_time(1000)		
	}		
	fn request_access_check() -> Weight {
		Weight::from_ref_time(1000)		
	}		
}

impl WeightInfo for () {
	fn create_guild() -> Weight {
		Weight::from_ref_time(1000)
	}
	fn create_free_role() -> Weight {
		Weight::from_ref_time(1000)
	}
	fn create_role_with_allowlist() -> Weight {
		Weight::from_ref_time(1000)
	}
	fn create_child_role() -> Weight {
		Weight::from_ref_time(1000)
	}
	fn create_unfiltered_role() -> Weight {
		Weight::from_ref_time(1000)
	}
	fn join_free_role() -> Weight {
		Weight::from_ref_time(1000)
	}
	fn join_child_role() -> Weight {
		Weight::from_ref_time(1000)
	}
	fn join_role_with_allowlist() -> Weight {
		Weight::from_ref_time(1000)
	}
	fn join_unfiltered_role() -> Weight {
		Weight::from_ref_time(1000)
	}
	fn leave() -> Weight {
		Weight::from_ref_time(1000)		
	}		
	fn request_access_check() -> Weight {
		Weight::from_ref_time(1000)		
	}		
}
