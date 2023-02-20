use super::*;
use crate::Pallet as Oracle;

use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use sp_core::Get;

const ACCOUNT: &str = "operator";
const SEED: u32 = 999;

benchmarks! {
    register_operator {
        let max_operators = <T as Config>::MaxOperators::get();
        let n in 1 .. <T as Config>::MaxOperators::get() - 1 => register_operators::<T>(n);
        let operator: T::AccountId = account(ACCOUNT, max_operators - 1, SEED);
    }: _(RawOrigin::Root, operator)
    verify {
        assert_eq!(Oracle::<T>::operators().len(), (n + 1) as usize);
    }
    deregister_operator {
        let max_operators = <T as Config>::MaxOperators::get();
        let n in 1 .. <T as Config>::MaxOperators::get();
        let operator = register_operators::<T>(n);
    }: _(RawOrigin::Root, operator)
    verify {
        assert_eq!(Oracle::<T>::operators().len(), (n - 1) as usize)
    }
    initiate_request {
        let n in 50 .. 1000;
        let caller: T::AccountId = whitelisted_caller();
        let operator: T::AccountId = account(ACCOUNT, 1, SEED);

        T::Currency::make_free_balance_be(
            &caller,
            <T::Currency as Currency<T::AccountId>>::Balance::from(100u32)
        );

        Oracle::<T>::register_operator(RawOrigin::Root.into(), operator)?;

        let data = vec![128; n as usize];
        let fee = T::Currency::minimum_balance();
        let callback = crate::mock::MockCallback::<T>::new();
    }: _(RawOrigin::Signed(caller), callback, data, fee)
    verify {
        assert_eq!(Oracle::<T>::request_identifier(), 1);
        assert_eq!(Oracle::<T>::next_operator(), 1);
    }

    impl_benchmark_test_suite!(Oracle, crate::mock::new_test_ext(), crate::mock::TestRuntime, extra = false);
}

fn register_operators<T: Config>(n: u32) -> T::AccountId {
    let operators: Vec<T::AccountId> = (0..n)
        .into_iter()
        .map(|i| account(ACCOUNT, i, SEED))
        .collect();

    let operator_0 = operators[0].clone();

    for operator in operators {
        Oracle::<T>::register_operator(RawOrigin::Root.into(), operator).unwrap();
    }

    operator_0
}
