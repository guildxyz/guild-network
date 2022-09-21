use super::*;

use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

benchmarks! {
    register_operator {
        let caller: T::AccountId = whitelisted_caller();
    }: _(RawOrigin::Signed(caller))
    verify {
        // TODO add some verification logic here?
    }

}

//impl_benchmark_test_suite!(Guild, crate::mock::new_test_ext(), crate::mock::Test);
