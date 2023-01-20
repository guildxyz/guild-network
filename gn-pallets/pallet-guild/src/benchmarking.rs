use super::*;

use crate::Pallet as Guild;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use gn_common::identities::IdentityWithAuth;
use gn_common::RequestData;
use pallet_oracle::Pallet as Oracle;
use sp_std::vec;

benchmarks! {
    create_guild {
        let r in 0..100;

        let caller: T::AccountId = whitelisted_caller();
        let guild_name = [0u8; 32];
        let guild_metadata = vec![155u8; 255];
        let mut roles = vec![];
        for i in 0 .. r {
            let role = ([i as u8; 32], (vec![123u8; 250], vec![vec![123u8; 250]]));
            roles.push(role);
        }
    }: _(RawOrigin::Signed(caller), guild_name, guild_metadata, roles)
    verify {
        let guild_id = Guild::<T>::guild_id(guild_name).unwrap();
        assert!(Guild::<T>::guild(guild_id).is_some());
    }

    register {
        let caller: T::AccountId = whitelisted_caller();
        let operator: T::AccountId = whitelisted_caller();

        T::Currency::make_free_balance_be(&caller, T::Currency::minimum_balance() + 2_000_000_000u32.into());
        Oracle::<T>::register_operator(RawOrigin::Signed(operator).into())?;
    }: _(
        RawOrigin::Signed(caller),
        RequestData::Register(vec![
            IdentityWithAuth::EvmChain([0; 20],[1; 65]),
            IdentityWithAuth::EvmChain([2; 20],[3; 65]),
            IdentityWithAuth::EvmChain([4; 20],[5; 65]),
            IdentityWithAuth::EvmChain([6; 20],[7; 65]),
            IdentityWithAuth::Discord(11234, ()),
            IdentityWithAuth::Telegram(9999999, ()),
        ])
    )
    verify {
        assert!(Oracle::<T>::request(0).is_some())
    }

    manage_role {
        let caller: T::AccountId = whitelisted_caller();
        let operator: T::AccountId = whitelisted_caller();

        let guild_name = [0u8; 32];
        let guild_metadata = vec![155u8; 255];
        let mut roles = vec![];
        for i in 0..20 {
            let role = ([i as u8; 32], (vec![123u8; 250], vec![vec![123u8; 250]]));
            roles.push(role);
        }

        T::Currency::make_free_balance_be(&caller, T::Currency::minimum_balance() + 2_000_000_000u32.into());
        Oracle::<T>::register_operator(RawOrigin::Signed(operator).into())?;
        Guild::<T>::create_guild(RawOrigin::Signed(caller.clone()).into(), guild_name, guild_metadata, roles)?;
    }: _(RawOrigin::Signed(caller), RequestData::ReqCheck { account: caller.clone(), guild: guild_name, role: [10u8; 32] })
    verify {
        assert!(Oracle::<T>::request(0).is_some());
    }
}
