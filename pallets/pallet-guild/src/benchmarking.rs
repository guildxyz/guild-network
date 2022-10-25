use super::*;

use crate::Pallet as Guild;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use pallet_chainlink::Pallet as Chainlink;

benchmarks! {
    create_guild {
        let caller: T::AccountId = whitelisted_caller();
    }: _(RawOrigin::Signed(caller), 69, 123)
    verify {
    }

    join_guild {
        let caller: T::AccountId = whitelisted_caller();
        let operator: T::AccountId = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, T::Currency::minimum_balance() + 2_000_000_000u32.into());
        Guild::<T>::create_guild(RawOrigin::Signed(caller.clone()).into(), 69, 123)?;
        Chainlink::<T>::register_operator(RawOrigin::Signed(operator.clone()).into())?;
    }: _(RawOrigin::Signed(caller), 69, sp_std::vec![128; 40])
    verify {
    }
}
