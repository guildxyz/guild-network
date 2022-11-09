use super::*;

use crate::Pallet as Guild;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use pallet_chainlink::Pallet as Chainlink;
use sp_std::vec;

benchmarks! {
    create_guild {
        let r in 0..100;

        let caller: T::AccountId = whitelisted_caller();
        let guild_name = [0u8; 32];
        let guild_metadata = vec![155u8; 255];
        let mut roles = vec![];
        for i in 0 .. r {
            let role = ([i as u8; 32], vec![123u8; 500]);
            roles.push(role);
        }
    }: _(RawOrigin::Signed(caller), guild_name, guild_metadata, roles)
    verify {
        let guild_id = Guild::<T>::guild_id(guild_name).unwrap();
        assert!(Guild::<T>::guild(guild_id).is_some());
    }

    join_guild {
        let caller: T::AccountId = whitelisted_caller();
        let operator: T::AccountId = whitelisted_caller();

        let guild_name = [0u8; 32];
        let guild_metadata = vec![155u8; 255];
        let mut roles = vec![];
        for i in 0..20 {
            let role = ([i as u8; 32], vec![123u8; 500]);
            roles.push(role);
        }

        T::Currency::make_free_balance_be(&caller, T::Currency::minimum_balance() + 2_000_000_000u32.into());
        Chainlink::<T>::register_operator(RawOrigin::Signed(operator).into())?;
        Guild::<T>::create_guild(RawOrigin::Signed(caller.clone()).into(), guild_name, guild_metadata, roles)?;
    }: _(RawOrigin::Signed(caller), guild_name, [10u8; 32], vec![0u8; 500], vec![0u8; 750])
    verify {
        assert!(Guild::<T>::join_request(0).is_some());
    }
}
