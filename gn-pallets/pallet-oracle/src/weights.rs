
//! Autogenerated weights for `pallet_oracle`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 19.0.0
//! DATE: 2023-04-27, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `turbineblade`, CPU: `AMD Ryzen 5 3600 6-Core Processor`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/gn-node
// benchmark
// pallet
// --chain
// dev
// --pallet
// pallet_oracle
// --extrinsic
// *
// --execution=wasm
// --wasm-execution=compiled
// --steps
// 50
// --repeat
// 20
// --output
// ./gn-pallets/pallet-oracle/src/weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

pub trait WeightInfo {
	fn register_operator(n: u32, ) -> Weight;
	fn deregister_operator(n: u32, ) -> Weight;
	fn activate_operator(n: u32, ) -> Weight;
	fn deactivate_operator(n: u32, ) -> Weight;
	fn initiate_request(n: u32, ) -> Weight;
}

/// Weight functions for `pallet_oracle`.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: Oracle NumRegisteredOperators (r:1 w:1)
	/// Proof Skipped: Oracle NumRegisteredOperators (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Oracle RegisteredOperators (r:1 w:1)
	/// Proof Skipped: Oracle RegisteredOperators (max_values: None, max_size: None, mode: Measured)
	/// The range of component `n` is `[1, 9]`.
	fn register_operator(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `77 + n * (25 ±0)`
		//  Estimated: `3120 + n * (50 ±0)`
		// Minimum execution time: 20_248 nanoseconds.
		Weight::from_parts(20_812_736, 3120)
			// Standard Error: 4_180
			.saturating_add(Weight::from_ref_time(106_003).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
			.saturating_add(Weight::from_proof_size(50).saturating_mul(n.into()))
	}
	/// Storage: Oracle RegisteredOperators (r:1 w:1)
	/// Proof Skipped: Oracle RegisteredOperators (max_values: None, max_size: None, mode: Measured)
	/// Storage: Oracle ActiveOperators (r:1 w:1)
	/// Proof Skipped: Oracle ActiveOperators (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Oracle NumRegisteredOperators (r:1 w:1)
	/// Proof Skipped: Oracle NumRegisteredOperators (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `n` is `[1, 9]`.
	fn deregister_operator(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `72 + n * (34 ±0)`
		//  Estimated: `3681 + n * (102 ±0)`
		// Minimum execution time: 20_779 nanoseconds.
		Weight::from_parts(22_478_047, 3681)
			// Standard Error: 10_930
			.saturating_add(Weight::from_ref_time(330_519).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
			.saturating_add(Weight::from_proof_size(102).saturating_mul(n.into()))
	}
	/// Storage: Oracle RegisteredOperators (r:1 w:0)
	/// Proof Skipped: Oracle RegisteredOperators (max_values: None, max_size: None, mode: Measured)
	/// Storage: Oracle ActiveOperators (r:1 w:1)
	/// Proof Skipped: Oracle ActiveOperators (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `n` is `[1, 10]`.
	fn activate_operator(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `95 + n * (66 ±0)`
		//  Estimated: `3124 + n * (136 ±0)`
		// Minimum execution time: 17_653 nanoseconds.
		Weight::from_parts(19_907_986, 3124)
			// Standard Error: 17_110
			.saturating_add(Weight::from_ref_time(514_061).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(Weight::from_proof_size(136).saturating_mul(n.into()))
	}
	/// Storage: Oracle RegisteredOperators (r:1 w:0)
	/// Proof Skipped: Oracle RegisteredOperators (max_values: None, max_size: None, mode: Measured)
	/// Storage: Oracle ActiveOperators (r:1 w:1)
	/// Proof Skipped: Oracle ActiveOperators (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `n` is `[1, 10]`.
	fn deactivate_operator(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `128 + n * (65 ±0)`
		//  Estimated: `3240 + n * (128 ±0)`
		// Minimum execution time: 20_057 nanoseconds.
		Weight::from_parts(21_006_615, 3240)
			// Standard Error: 6_533
			.saturating_add(Weight::from_ref_time(347_594).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(Weight::from_proof_size(128).saturating_mul(n.into()))
	}
	/// Storage: Oracle ActiveOperators (r:1 w:0)
	/// Proof Skipped: Oracle ActiveOperators (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Oracle NextOperator (r:1 w:1)
	/// Proof Skipped: Oracle NextOperator (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Oracle NextRequestIdentifier (r:1 w:1)
	/// Proof Skipped: Oracle NextRequestIdentifier (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Oracle Requests (r:0 w:1)
	/// Proof Skipped: Oracle Requests (max_values: None, max_size: None, mode: Measured)
	/// The range of component `n` is `[50, 1000]`.
	fn initiate_request(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `153`
		//  Estimated: `2097`
		// Minimum execution time: 24_356 nanoseconds.
		Weight::from_parts(25_390_559, 2097)
			// Standard Error: 42
			.saturating_add(Weight::from_ref_time(395).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
}

impl WeightInfo for () {
	/// Storage: Oracle NumRegisteredOperators (r:1 w:1)
	/// Proof Skipped: Oracle NumRegisteredOperators (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Oracle RegisteredOperators (r:1 w:1)
	/// Proof Skipped: Oracle RegisteredOperators (max_values: None, max_size: None, mode: Measured)
	/// The range of component `n` is `[1, 9]`.
	fn register_operator(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `77 + n * (25 ±0)`
		//  Estimated: `3120 + n * (50 ±0)`
		// Minimum execution time: 20_248 nanoseconds.
		Weight::from_parts(20_812_736, 3120)
			// Standard Error: 4_180
			.saturating_add(Weight::from_ref_time(106_003).saturating_mul(n.into()))
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(2))
			.saturating_add(Weight::from_proof_size(50).saturating_mul(n.into()))
	}
	/// Storage: Oracle RegisteredOperators (r:1 w:1)
	/// Proof Skipped: Oracle RegisteredOperators (max_values: None, max_size: None, mode: Measured)
	/// Storage: Oracle ActiveOperators (r:1 w:1)
	/// Proof Skipped: Oracle ActiveOperators (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Oracle NumRegisteredOperators (r:1 w:1)
	/// Proof Skipped: Oracle NumRegisteredOperators (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `n` is `[1, 9]`.
	fn deregister_operator(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `72 + n * (34 ±0)`
		//  Estimated: `3681 + n * (102 ±0)`
		// Minimum execution time: 20_779 nanoseconds.
		Weight::from_parts(22_478_047, 3681)
			// Standard Error: 10_930
			.saturating_add(Weight::from_ref_time(330_519).saturating_mul(n.into()))
			.saturating_add(RocksDbWeight::get().reads(3))
			.saturating_add(RocksDbWeight::get().writes(3))
			.saturating_add(Weight::from_proof_size(102).saturating_mul(n.into()))
	}
	/// Storage: Oracle RegisteredOperators (r:1 w:0)
	/// Proof Skipped: Oracle RegisteredOperators (max_values: None, max_size: None, mode: Measured)
	/// Storage: Oracle ActiveOperators (r:1 w:1)
	/// Proof Skipped: Oracle ActiveOperators (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `n` is `[1, 10]`.
	fn activate_operator(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `95 + n * (66 ±0)`
		//  Estimated: `3124 + n * (136 ±0)`
		// Minimum execution time: 17_653 nanoseconds.
		Weight::from_parts(19_907_986, 3124)
			// Standard Error: 17_110
			.saturating_add(Weight::from_ref_time(514_061).saturating_mul(n.into()))
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(1))
			.saturating_add(Weight::from_proof_size(136).saturating_mul(n.into()))
	}
	/// Storage: Oracle RegisteredOperators (r:1 w:0)
	/// Proof Skipped: Oracle RegisteredOperators (max_values: None, max_size: None, mode: Measured)
	/// Storage: Oracle ActiveOperators (r:1 w:1)
	/// Proof Skipped: Oracle ActiveOperators (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `n` is `[1, 10]`.
	fn deactivate_operator(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `128 + n * (65 ±0)`
		//  Estimated: `3240 + n * (128 ±0)`
		// Minimum execution time: 20_057 nanoseconds.
		Weight::from_parts(21_006_615, 3240)
			// Standard Error: 6_533
			.saturating_add(Weight::from_ref_time(347_594).saturating_mul(n.into()))
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(1))
			.saturating_add(Weight::from_proof_size(128).saturating_mul(n.into()))
	}
	/// Storage: Oracle ActiveOperators (r:1 w:0)
	/// Proof Skipped: Oracle ActiveOperators (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Oracle NextOperator (r:1 w:1)
	/// Proof Skipped: Oracle NextOperator (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Oracle NextRequestIdentifier (r:1 w:1)
	/// Proof Skipped: Oracle NextRequestIdentifier (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Oracle Requests (r:0 w:1)
	/// Proof Skipped: Oracle Requests (max_values: None, max_size: None, mode: Measured)
	/// The range of component `n` is `[50, 1000]`.
	fn initiate_request(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `153`
		//  Estimated: `2097`
		// Minimum execution time: 24_356 nanoseconds.
		Weight::from_parts(25_390_559, 2097)
			// Standard Error: 42
			.saturating_add(Weight::from_ref_time(395).saturating_mul(n.into()))
			.saturating_add(RocksDbWeight::get().reads(3))
			.saturating_add(RocksDbWeight::get().writes(3))
	}
}
