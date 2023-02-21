use super::*;
use crate::Pallet as Oracle;

use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::dispatch::{
    DispatchResultWithPostInfo, PostDispatchInfo, UnfilteredDispatchable,
};
use frame_support::pallet_prelude::Pays;
use frame_support::traits::{Currency, Get};
use frame_system::RawOrigin;
use parity_scale_codec::{Decode, Encode, EncodeLike};
use scale_info::TypeInfo;
use sp_std::{vec, vec::Vec};

const ACCOUNT: &str = "operator";
const SEED: u32 = 999;

benchmarks! {
    register_operator {
        let max_operators = <T as Config>::MaxOperators::get();
        let n in 1 .. <T as Config>::MaxOperators::get() - 1 => register_operators::<T>(n);
        let operator: T::AccountId = account(ACCOUNT, max_operators - 1, SEED);
    }: _(RawOrigin::Root, operator.clone())
    verify {
        assert!(Oracle::<T>::operator(operator).is_some());
    }
    deregister_operator {
        let max_operators = <T as Config>::MaxOperators::get();
        let n in 1 .. <T as Config>::MaxOperators::get() - 1;
        let operators = register_operators::<T>(n);
    }: _(RawOrigin::Root, operators[0].clone())
    verify {
        assert!(Oracle::<T>::operator(operators[0].clone()).is_none());
    }
    activate_operator {
        let n in 1 .. <T as Config>::MaxOperators::get();
        let mut operators = register_operators::<T>(n);
        for operator in operators.iter().skip(1) {
            Oracle::<T>::activate_operator(RawOrigin::Signed(operator.clone()).into()).unwrap();
        }
    }: _(RawOrigin::Signed(operators[0].clone()))
    verify {
        operators.sort();
        assert_eq!(Oracle::<T>::active_operators(), operators);
    }
    deactivate_operator {
        let n in 1 .. <T as Config>::MaxOperators::get();
        let operators = register_operators::<T>(n);
        for operator in &operators {
            Oracle::<T>::activate_operator(RawOrigin::Signed(operator.clone()).into()).unwrap();
        }
    }: _(RawOrigin::Signed(operators[0].clone()))
    verify {
        assert!(!Oracle::<T>::active_operators().contains(&operators[0]));
    }
    initiate_request {
        let n in 50 .. 1000;
        let caller: T::AccountId = whitelisted_caller();
        let operator: T::AccountId = account(ACCOUNT, 1, SEED);

        T::Currency::make_free_balance_be(
            &caller,
            <T::Currency as Currency<T::AccountId>>::Balance::from(100u32)
        );

        Oracle::<T>::register_operator(RawOrigin::Root.into(), operator.clone())?;
        Oracle::<T>::activate_operator(RawOrigin::Signed(operator.clone()).into())?;

        let data = vec![128; n as usize];
        let fee = T::Currency::minimum_balance();
        let callback = MockCallback::<T>::test();
    }: _(RawOrigin::Signed(caller), callback, data, fee)
    verify {
        assert_eq!(Oracle::<T>::request_identifier(), 1);
        assert_eq!(Oracle::<T>::next_operator(), 1);
    }

    impl_benchmark_test_suite!(Oracle, crate::mock::new_test_ext(), crate::mock::TestRuntime, extra = false);
}

fn register_operators<T: Config>(n: u32) -> Vec<T::AccountId> {
    let operators: Vec<T::AccountId> = (0..n).map(|i| account(ACCOUNT, i, SEED)).collect();

    for operator in &operators {
        Oracle::<T>::register_operator(RawOrigin::Root.into(), operator.clone()).unwrap();
    }

    operators
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, TypeInfo, Encode, Decode)]
pub struct MockCallback<T>(pub core::marker::PhantomData<T>);

impl<T> EncodeLike<()> for MockCallback<T> {}

impl<T> CallbackWithParameter for MockCallback<T> {
    fn with_result(&self, result: Vec<u8>) -> Option<Self> {
        if result == [0, 0] {
            None
        } else {
            Some(Self(core::marker::PhantomData))
        }
    }
}

impl<T: frame_system::Config> UnfilteredDispatchable for MockCallback<T> {
    type RuntimeOrigin = <T as frame_system::Config>::RuntimeOrigin;
    fn dispatch_bypass_filter(self, _origin: Self::RuntimeOrigin) -> DispatchResultWithPostInfo {
        Ok(PostDispatchInfo {
            actual_weight: None,
            pays_fee: Pays::No,
        })
    }
}

impl<T: Config> MockCallback<T> {
    pub fn test() -> <T as Config>::Callback {
        let mut enc = vec![9];
        enc.extend(vec![1u8, 2, 3].encode());
        Decode::decode(&mut &enc[..]).unwrap()
    }
}
