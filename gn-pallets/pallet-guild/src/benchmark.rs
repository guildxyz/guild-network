use super::*;
use crate::Pallet as Guild;
use pallet_guild_identity::Pallet as GuildIdentity;
use pallet_oracle::Pallet as Oracle;

use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::assert_ok;
use frame_support::traits::Get;
use frame_system::RawOrigin;
use gn_common::filter::{Guild as GuildFilter, Logic as FilterLogic};
use gn_common::merkle::Proof as MerkleProof;
use gn_common::{GuildName, RoleName};
use sp_std::vec;

const ACCOUNT: &str = "account";
const SEED: u32 = 999;

benchmarks! {
    create_guild {
        let n in 0 .. <T as Config>::MaxSerializedLen::get();

        let user: T::AccountId = whitelisted_caller();
        let guild_name = [0u8; 32];
        let metadata = vec![0u8; n as usize];
    }: _(RawOrigin::Signed(user), guild_name, metadata)
    verify {
        assert!(Guild::<T>::guild_id(guild_name).is_some());
    }
    create_free_role {
        let user: T::AccountId = whitelisted_caller();
        let guild_name = [0u8; 32];
        let role_name = [0u8; 32];
        init_guild::<T>(&user, guild_name);
    }: _(RawOrigin::Signed(user), guild_name, role_name)
    verify {
        let guild_id = Guild::<T>::guild_id(guild_name).unwrap();
        assert!(Guild::<T>::role_id(guild_id, role_name).is_some());
    }

    create_role_with_allowlist {
        let n in 1 .. <T as Config>::MaxAllowlistLen::get();
        let r in 0 .. <T as Config>::MaxReqsPerRole::get();
        let s in 0 .. <T as Config>::MaxSerializedLen::get();
        let logic = vec![100u8; s as usize];
        let req = vec![200u8; s as usize];
        let serialized_requirements = (vec![req; r as usize], logic);

        let user: T::AccountId = whitelisted_caller();
        let guild_name = [0u8; 32];
        let role_name = [0u8; 32];
        init_guild::<T>(&user, guild_name);

        let allowlist = vec![account(ACCOUNT, 123, SEED); n as usize];
    }: _(RawOrigin::Signed(user), guild_name, role_name, allowlist, FilterLogic::And, Some(serialized_requirements))
    verify {
        let guild_id = Guild::<T>::guild_id(guild_name).unwrap();
        assert!(Guild::<T>::role_id(guild_id, role_name).is_some());
    }

    create_child_role {
        let r in 0 .. <T as Config>::MaxReqsPerRole::get();
        let s in 0 .. <T as Config>::MaxSerializedLen::get();
        let logic = vec![100u8; s as usize];
        let req = vec![200u8; s as usize];
        let serialized_requirements = (vec![req; r as usize], logic);

        let user: T::AccountId = whitelisted_caller();
        let guild_name = [0u8; 32];
        let role_name = [0u8; 32];
        let free_role_name = [1u8; 32];
        init_guild::<T>(&user, guild_name);
        Guild::<T>::create_free_role(
            RawOrigin::Signed(user.clone()).into(),
            guild_name,
            free_role_name,
        ).unwrap();

        let filter = GuildFilter {
            name: guild_name,
            role: Some(free_role_name),
        };
    }: _(RawOrigin::Signed(user), guild_name, role_name, filter, FilterLogic::And, Some(serialized_requirements))
    verify {
        let guild_id = Guild::<T>::guild_id(guild_name).unwrap();
        assert!(Guild::<T>::role_id(guild_id, role_name).is_some());
    }

    create_unfiltered_role {
        let r in 0 .. <T as Config>::MaxReqsPerRole::get();
        let s in 0 .. <T as Config>::MaxSerializedLen::get();
        let logic = vec![100u8; s as usize];
        let req = vec![200u8; s as usize];
        let serialized_requirements = (vec![req; r as usize], logic);

        let user: T::AccountId = whitelisted_caller();
        let guild_name = [0u8; 32];
        let role_name = [0u8; 32];
        init_guild::<T>(&user, guild_name);
    }: _(RawOrigin::Signed(user), guild_name, role_name, serialized_requirements)
    verify {
        let guild_id = Guild::<T>::guild_id(guild_name).unwrap();
        assert!(Guild::<T>::role_id(guild_id, role_name).is_some());
    }

    join_free_role {
        let guild_name = [0u8; 32];
        let role_name = [0u8; 32];
        let user: T::AccountId = whitelisted_caller();

        init_guild::<T>(&user, guild_name);
        assert_ok!(Guild::<T>::create_free_role(RawOrigin::Signed(user.clone()).into(), guild_name, role_name));
        assert_ok!(GuildIdentity::<T>::register(RawOrigin::Signed(user.clone()).into()));
    }: _(RawOrigin::Signed(user.clone()), guild_name, role_name)
    verify {
        membership_check::<T>(guild_name, role_name, user);
    }

    join_child_role {
        let logic = vec![100u8; <T as Config>::MaxSerializedLen::get() as usize];
        let req = vec![200u8; <T as Config>::MaxSerializedLen::get() as usize];
        let serialized_requirements = (vec![req; <T as Config>::MaxReqsPerRole::get() as usize], logic);

        let guild_name = [0u8; 32];
        let parent_role_name = [0u8; 32];
        let child_role_name = [1u8; 32];

        // oracle + identity reg
        let user: T::AccountId = whitelisted_caller();
        let operator: T::AccountId = account(ACCOUNT, 123, SEED);
        oracle_init_and_register::<T>(&user, &operator);
        // create guild and parent role
        init_guild::<T>(&user, guild_name);
        assert_ok!(Guild::<T>::create_free_role(RawOrigin::Signed(user.clone()).into(), guild_name, parent_role_name));
        // join parent role
        assert_ok!(Guild::<T>::join_free_role(RawOrigin::Signed(user.clone()).into(), guild_name, parent_role_name));
        // create child role
        let filter = GuildFilter {
            name: guild_name,
            role: Some(parent_role_name),
        };
        assert_ok!(Guild::<T>::create_child_role(
            RawOrigin::Signed(
                user.clone()).into(),
                guild_name,
                child_role_name,
                filter,
                FilterLogic::And,
                Some(serialized_requirements)
        ));
    }: _(RawOrigin::Signed(user.clone()), guild_name, child_role_name)
    verify {
        assert_ok!(Guild::<T>::callback(RawOrigin::Signed(operator).into(), 0, true));
        membership_check::<T>(guild_name, child_role_name, user);
    }

    join_unfiltered_role {
        let logic = vec![100u8; <T as Config>::MaxSerializedLen::get() as usize];
        let req = vec![200u8; <T as Config>::MaxSerializedLen::get() as usize];
        let serialized_requirements = (vec![req; <T as Config>::MaxReqsPerRole::get() as usize], logic);

        // oracle + identity reg
        let user: T::AccountId = whitelisted_caller();
        let operator: T::AccountId = account(ACCOUNT, 123, SEED);
        oracle_init_and_register::<T>(&user, &operator);

        // guild
        let guild_name = [0u8; 32];
        let role_name = [0u8; 32];
        init_guild::<T>(&user, guild_name);

        assert_ok!(Guild::<T>::create_unfiltered_role(
            RawOrigin::Signed(user.clone()).into(),
            guild_name,
            role_name,
            serialized_requirements,
        ));
    }: _(RawOrigin::Signed(user.clone()), guild_name, role_name)
    verify {
        assert_ok!(Guild::<T>::callback(RawOrigin::Signed(operator).into(), 0, true));
        membership_check::<T>(guild_name, role_name, user);
    }

    join_role_with_allowlist {
        let logic = vec![100u8; <T as Config>::MaxSerializedLen::get() as usize];
        let req = vec![200u8; <T as Config>::MaxSerializedLen::get() as usize];
        let serialized_requirements = (vec![req; <T as Config>::MaxReqsPerRole::get() as usize], logic);

        // oracle + identity reg
        let user: T::AccountId = whitelisted_caller();
        let operator: T::AccountId = account(ACCOUNT, 123, SEED);
        oracle_init_and_register::<T>(&user, &operator);

        // guild
        let guild_name = [0u8; 32];
        let role_name = [0u8; 32];
        init_guild::<T>(&user, guild_name);
        let mut allowlist = vec![account(ACCOUNT, 10, SEED); (<T as Config>::MaxAllowlistLen::get() - 1) as usize];
        allowlist.push(user.clone());

        assert_ok!(Guild::<T>::create_role_with_allowlist(
            RawOrigin::Signed(user.clone()).into(),
            guild_name,
            role_name,
            allowlist.clone(),
            FilterLogic::And,
            Some(serialized_requirements),
        ));

        // proof to the last element
        let proof = MerkleProof::new(&allowlist, allowlist.len() - 1);
    }: _(RawOrigin::Signed(user.clone()), guild_name, role_name, proof)
    verify {
        assert_ok!(Guild::<T>::callback(RawOrigin::Signed(operator).into(), 0, true));
        membership_check::<T>(guild_name, role_name, user);
    }

    leave {
        let user: T::AccountId = whitelisted_caller();

        assert_ok!(GuildIdentity::<T>::register(
            RawOrigin::Signed(user.clone()).into(),
        ));

        let guild_name = [0u8; 32];
        let role_name = [0u8; 32];
        init_guild::<T>(&user, guild_name);
        assert_ok!(Guild::<T>::create_free_role(
            RawOrigin::Signed(user.clone()).into(),
            guild_name,
            role_name,
        ));

        assert_ok!(Guild::<T>::join_free_role(
            RawOrigin::Signed(user.clone()).into(),
            guild_name,
            role_name,
        ));
    }: _(RawOrigin::Signed(user.clone()), guild_name, role_name)
    verify {
        let guild_id = Guild::<T>::guild_id(guild_name).unwrap();
        let role_id = Guild::<T>::role_id(guild_id, role_name).unwrap();
        assert!(Guild::<T>::member(role_id, user).is_none());
    }

    request_access_check {
        let r = <T as Config>::MaxReqsPerRole::get() as usize;
        let s = <T as Config>::MaxSerializedLen::get() as usize;
        let logic = vec![100u8; s];
        let req = vec![200u8; s];
        let serialized_requirements = (vec![req; r], logic);

        let user: T::AccountId = whitelisted_caller();
        let keeper: T::AccountId = account(ACCOUNT, 123, SEED);
        let operator: T::AccountId = account(ACCOUNT, 222, SEED);
        oracle_init_and_register::<T>(&user, &operator);

        let guild_name = [0u8; 32];
        let role_name = [0u8; 32];
        init_guild::<T>(&user, guild_name);

        assert_ok!(Guild::<T>::create_unfiltered_role(
            RawOrigin::Signed(user.clone()).into(),
            guild_name,
            role_name,
            serialized_requirements,
        ));

        assert_ok!(Guild::<T>::join_unfiltered_role(
            RawOrigin::Signed(user.clone()).into(),
            guild_name,
            role_name,
        ));

        assert_ok!(Guild::<T>::callback(RawOrigin::Signed(operator.clone()).into(), 0, true));
    }: _(RawOrigin::Signed(keeper.clone()), user.clone(), guild_name, role_name)
    verify {
        assert_ok!(Guild::<T>::callback(RawOrigin::Signed(operator).into(), 1, true));
        membership_check::<T>(guild_name, role_name, user);
    }

    impl_benchmark_test_suite!(Guild, crate::mock::new_test_ext(), crate::mock::TestRuntime, extra = false);
}

fn init_guild<T: Config>(user: &T::AccountId, guild_name: [u8; 32]) {
    frame_system::Pallet::<T>::set_block_number(<T as frame_system::Config>::BlockNumber::from(
        1u32,
    ));
    let metadata = vec![0u8; <T as Config>::MaxSerializedLen::get() as usize];
    assert_ok!(Guild::<T>::create_guild(
        RawOrigin::Signed(user.clone()).into(),
        guild_name,
        metadata,
    ));
}

fn oracle_init_and_register<T: Config>(user: &T::AccountId, operator: &T::AccountId) {
    assert_ok!(Oracle::<T>::register_operator(
        RawOrigin::Root.into(),
        operator.clone(),
    ));
    assert_ok!(Oracle::<T>::activate_operator(
        RawOrigin::Signed(operator.clone()).into(),
    ));

    assert_ok!(GuildIdentity::<T>::register(
        RawOrigin::Signed(user.clone()).into(),
    ));
}

fn membership_check<T: Config>(guild_name: GuildName, role_name: RoleName, user: T::AccountId) {
    let guild_id = Guild::<T>::guild_id(guild_name).unwrap();
    let role_id = Guild::<T>::role_id(guild_id, role_name).unwrap();
    assert!(Guild::<T>::member(role_id, user).is_some());
}
