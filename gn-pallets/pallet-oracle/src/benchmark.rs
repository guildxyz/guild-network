use super::*;
use crate::Pallet as Oracle;
use crate::mock::MockCallback;

use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use frame_support::dispatch::{
    DispatchResultWithPostInfo, PostDispatchInfo, UnfilteredDispatchable,
};
use frame_support::pallet_prelude::Pays;
use frame_support::traits::Currency;
use parity_scale_codec::{Encode, Decode, EncodeLike};
use scale_info::TypeInfo;

const MAX_OPERATORS: u32 = 100;

benchmarks! {
    register_operator {
        let n in 0 .. MAX_OPERATORS - 1 => register_operators::<T>(n);
        let operator: T::AccountId = account("operator", MAX_OPERATORS - 1, 999);
    }: _(RawOrigin::Root, operator)
    verify {
        assert_eq!(Oracle::<T>::operators().len(), MAX_OPERATORS as usize)
    }
    deregister_operator {
        let n in 0 .. MAX_OPERATORS => register_operators::<T>(n);
        let operator: T::AccountId = account("operator", MAX_OPERATORS - 1, 999);
    }: _(RawOrigin::Root, operator)
    verify {
        assert_eq!(Oracle::<T>::operators().len(), MAX_OPERATORS as usize)
    }
    initiate_request {
        let n in 50 .. 1000;
        let caller: T::AccountId = whitelisted_caller();
        let operator: T::AccountId = account("operator", 1, 123);

        Oracle::<T>::register_operator(RawOrigin::Root.into(), operator)?;

        let data = vec![128; n as usize];
        let fee = T::Currency::minimum_balance();
        let callback = MockCallback(std::marker::PhantomData);
    }: _(RawOrigin::Signed(caller), callback, data, fee)
    verify {
        assert_eq!(Oracle::<T>::request_identifier(), 1);
        assert_eq!(Oracle::<T>::next_operator(), 1);
    }
}

fn register_operators<T: Config>(n: u32) {
    for i in 0..n {
        let operator: T::AccountId = account("operator", i, 999);
        Oracle::<T>::register_operator(RawOrigin::Root.into(), operator).unwrap();
    }
}

pub struct Pallet<T: Config>(crate::Pallet<T>);
pub trait Config: crate::Config {}
