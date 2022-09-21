use super::*;

use crate::Pallet as Chainlink;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

benchmarks! {
    register_operator {
        let caller: T::AccountId = whitelisted_caller();
    }: _(RawOrigin::Signed(caller))
    verify {
        assert_eq!(1, 1);
    }

}

//impl_benchmark_test_suite!(Guild, crate::mock::new_test_ext(), crate::mock::Test);
