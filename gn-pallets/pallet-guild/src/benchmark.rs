use super::*;
use crate::Pallet as Guild;

use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::traits::Get;
use frame_system::RawOrigin;
use gn_common::filter::{Guild as GuildFilter, Logic as FilterLogic};
use gn_common::identity::*;
use gn_common::merkle::Proof as MerkleProof;
use sp_std::vec;

const ACCOUNT: &str = "account";
const SEED: u32 = 999;

benchmarks! {
    register {
        let caller: T::AccountId = whitelisted_caller();
        let (identity, signature) = id_with_auth::<T>();
        let identity_with_auth = IdentityWithAuth::Ecdsa(identity, signature);
        let index = 1;
    }: _(RawOrigin::Signed(caller.clone()), identity_with_auth, index)
    verify {
        assert_eq!(Guild::<T>::user_data(caller, index), Some(identity));
    }
    create_guild {
        let n in 0 .. <T as Config>::MaxSerializedLen::get();

        let caller: T::AccountId = whitelisted_caller();
        let guild_name = [0u8; 32];
        let metadata = vec![0u8; n as usize];
    }: _(RawOrigin::Signed(caller), guild_name, metadata)
    verify {
        assert!(Guild::<T>::guild_id(guild_name).is_some());
    }
    create_free_role {
        let caller: T::AccountId = whitelisted_caller();
        let guild_name = [0u8; 32];
        let role_name = [0u8; 32];
        init_guild::<T>(&caller, guild_name);
    }: _(RawOrigin::Signed(caller), guild_name, role_name)
    verify {
        let guild_id = Guild::<T>::guild_id(guild_name).unwrap();
        assert!(Guild::<T>::role_id(guild_id, role_name).is_some());
    }

    create_role_with_allowlist {
        let n in 1 .. <T as Config>::MaxAllowlistLen::get();
        let r in 0 .. <T as Config>::MaxReqsPerRole::get();
        let s in 0 .. <T as Config>::MaxSerializedLen::get();

        let caller: T::AccountId = whitelisted_caller();
        let guild_name = [0u8; 32];
        let role_name = [0u8; 32];
        init_guild::<T>(&caller, guild_name);

        let allowlist = vec![Identity::Other([0u8; 64]); n as usize];
        let logic = vec![100u8; s as usize];
        let req = vec![200u8; s as usize];
        let serialized_requirements = (vec![req; r as usize], logic);
    }: _(RawOrigin::Signed(caller), guild_name, role_name, allowlist, FilterLogic::And, Some(serialized_requirements))
    verify {
        let guild_id = Guild::<T>::guild_id(guild_name).unwrap();
        assert!(Guild::<T>::role_id(guild_id, role_name).is_some());
    }

    create_child_role {
        let r in 0 .. <T as Config>::MaxReqsPerRole::get();
        let s in 0 .. <T as Config>::MaxSerializedLen::get();

        let caller: T::AccountId = whitelisted_caller();
        let guild_name = [0u8; 32];
        let role_name = [0u8; 32];
        let free_role_name = [1u8; 32];
        init_guild::<T>(&caller, guild_name);
        Guild::<T>::create_free_role(
            RawOrigin::Signed(caller.clone()).into(),
            guild_name,
            free_role_name,
        ).unwrap();

        let logic = vec![100u8; s as usize];
        let req = vec![200u8; s as usize];
        let serialized_requirements = (vec![req; r as usize], logic);
        let filter = GuildFilter {
            name: guild_name,
            role: Some(free_role_name),
        };
    }: _(RawOrigin::Signed(caller), guild_name, role_name, filter, FilterLogic::And, Some(serialized_requirements))
    verify {
        let guild_id = Guild::<T>::guild_id(guild_name).unwrap();
        assert!(Guild::<T>::role_id(guild_id, role_name).is_some());
    }

    create_unfiltered_role {
        let r in 0 .. <T as Config>::MaxReqsPerRole::get();
        let s in 0 .. <T as Config>::MaxSerializedLen::get();

        let caller: T::AccountId = whitelisted_caller();
        let guild_name = [0u8; 32];
        let role_name = [0u8; 32];
        init_guild::<T>(&caller, guild_name);
        let logic = vec![100u8; s as usize];
        let req = vec![200u8; s as usize];
        let serialized_requirements = (vec![req; r as usize], logic);
    }: _(RawOrigin::Signed(caller), guild_name, role_name, serialized_requirements)
    verify {
        let guild_id = Guild::<T>::guild_id(guild_name).unwrap();
        assert!(Guild::<T>::role_id(guild_id, role_name).is_some());
    }

    join {
        let n = <T as Config>::MaxAllowlistLen::get() as usize;

        // identity
        let caller: T::AccountId = whitelisted_caller();
        let (identity, signature) = id_with_auth::<T>();
        let identity_with_auth = IdentityWithAuth::Ecdsa(identity, signature);
        Guild::<T>::register(
            RawOrigin::Signed(caller.clone()).into(),
            identity_with_auth,
            0,
        ).unwrap();

        // guild
        let guild_name = [0u8; 32];
        let role_name = [0u8; 32];
        init_guild::<T>(&caller, guild_name);
        let mut allowlist = vec![Identity::Address20([0u8; 20]); n - 1];
        allowlist.push(identity);

        Guild::<T>::create_role_with_allowlist(
            RawOrigin::Signed(caller.clone()).into(),
            guild_name,
            role_name,
            allowlist.clone(),
            FilterLogic::And,
            None,
        ).unwrap();

        // proof to the last element
        let proof = MerkleProof::new(&allowlist, n - 1, 0);

    }: _(RawOrigin::Signed(caller.clone()), guild_name, role_name, Some(proof))
    verify {
        let guild_id = Guild::<T>::guild_id(guild_name).unwrap();
        let role_id = Guild::<T>::role_id(guild_id, role_name).unwrap();
        assert!(Guild::<T>::member(role_id, caller).is_some());
    }

    leave {
        let caller: T::AccountId = whitelisted_caller();
        let (identity, signature) = id_with_auth::<T>();
        let identity_with_auth = IdentityWithAuth::Ecdsa(identity, signature);

        Guild::<T>::register(
            RawOrigin::Signed(caller.clone()).into(),
            identity_with_auth,
            0,
        ).unwrap();

        let guild_name = [0u8; 32];
        let role_name = [0u8; 32];
        init_guild::<T>(&caller, guild_name);
        Guild::<T>::create_free_role(
            RawOrigin::Signed(caller.clone()).into(),
            guild_name,
            role_name,
        ).unwrap();

        Guild::<T>::join(
            RawOrigin::Signed(caller.clone()).into(),
            guild_name,
            role_name,
            None,
        ).unwrap();
    }: _(RawOrigin::Signed(caller.clone()), guild_name, role_name)
    verify {
        let guild_id = Guild::<T>::guild_id(guild_name).unwrap();
        let role_id = Guild::<T>::role_id(guild_id, role_name).unwrap();
        assert!(Guild::<T>::member(role_id, caller).is_none());
    }

    request_oracle_check {
        let r = <T as Config>::MaxReqsPerRole::get() as usize;
        let s = <T as Config>::MaxSerializedLen::get() as usize;

        let caller: T::AccountId = whitelisted_caller();
        let keeper: T::AccountId = account(ACCOUNT, 123, SEED);
        let operator: T::AccountId = account(ACCOUNT, 222, SEED);
        let (identity, signature) = id_with_auth::<T>();
        let identity_with_auth = IdentityWithAuth::Ecdsa(identity, signature);

        pallet_oracle::Pallet::<T>::register_operator(
            RawOrigin::Root.into(),
            operator.clone()
        ).unwrap();
        pallet_oracle::Pallet::<T>::activate_operator(
            RawOrigin::Signed(operator.clone()).into(),
        ).unwrap();
        Guild::<T>::register(
            RawOrigin::Signed(caller.clone()).into(),
            identity_with_auth,
            0,
        ).unwrap();

        let guild_name = [0u8; 32];
        let role_name = [0u8; 32];
        init_guild::<T>(&caller, guild_name);

        let logic = vec![100u8; s];
        let req = vec![200u8; s];
        let serialized_requirements = (vec![req; r], logic);

        Guild::<T>::create_unfiltered_role(
            RawOrigin::Signed(caller.clone()).into(),
            guild_name,
            role_name,
            serialized_requirements,
        ).unwrap();

        Guild::<T>::join(
            RawOrigin::Signed(caller.clone()).into(),
            guild_name,
            role_name,
            None,
        ).unwrap();

        pallet_oracle::Pallet::<T>::callback(
            RawOrigin::Signed(operator).into(),
            0,
            vec![1]
        ).unwrap();

    }: _(RawOrigin::Signed(keeper.clone()), caller.clone(), guild_name, role_name)
    verify {
        let guild_id = Guild::<T>::guild_id(guild_name).unwrap();
        let role_id = Guild::<T>::role_id(guild_id, role_name).unwrap();
        assert!(Guild::<T>::member(role_id, caller).is_some());
    }

    impl_benchmark_test_suite!(Guild, crate::mock::new_test_ext(), crate::mock::TestRuntime, extra = false);
}

fn init_guild<T: Config>(caller: &T::AccountId, guild_name: [u8; 32]) {
    frame_system::Pallet::<T>::set_block_number(<T as frame_system::Config>::BlockNumber::from(
        1u32,
    ));
    let metadata = vec![0u8; <T as Config>::MaxSerializedLen::get() as usize];
    Guild::<T>::create_guild(
        RawOrigin::Signed(caller.clone()).into(),
        guild_name,
        metadata,
    )
    .unwrap();
}

const ADDRESS: [u8; 20] = [
    181, 107, 240, 94, 75, 219, 191, 204, 187, 168, 13, 127, 220, 79, 13, 235, 246, 21, 213, 11,
];

const SIGNATURE: [u8; 65] = [
    252, 125, 173, 220, 20, 148, 251, 98, 222, 103, 168, 18, 25, 200, 32, 44, 130, 113, 16, 110,
    44, 102, 249, 87, 225, 146, 239, 99, 61, 41, 59, 116, 75, 60, 155, 227, 103, 131, 188, 167,
    198, 47, 72, 62, 166, 146, 182, 134, 9, 159, 28, 76, 188, 7, 20, 189, 106, 78, 47, 114, 17, 86,
    201, 32, 1,
];

fn id_with_auth<T: Config>() -> (Identity, EcdsaSignature) {
    (Identity::Address20(ADDRESS), EcdsaSignature(SIGNATURE))
}
