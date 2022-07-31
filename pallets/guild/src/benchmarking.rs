//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as Guild;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

benchmarks! {
	create_guild {
		let caller: T::AccountId = whitelisted_caller();
	}: _(RawOrigin::Signed(caller), 69)
	verify {
		assert_eq!(1, 1);
	}

	join_guild {
		let caller: T::AccountId = whitelisted_caller();
		Guild::<T>::create_guild(RawOrigin::Signed(caller.clone()).into(), 69);
	}: _(RawOrigin::Signed(caller), 69)
	verify {
		assert_eq!(1, 1);
	}

	impl_benchmark_test_suite!(Guild, crate::mock::new_test_ext(), crate::mock::Test);
}
