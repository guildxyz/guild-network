use super::*;
use crate::Pallet as Guild;

use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use gn_common::identity::*;
use sp_core::Pair as PairT;

const ACCOUNT: &str = "account";
const SEED: u32 = 999;

benchmarks! {
    register {
        let caller: T::AccountId = whitelisted_caller();

        let seed = [12u8; 32];
        let msg = gn_common::utils::verification_msg(&caller);
        let keypair = sp_core::ecdsa::Pair::from_seed_slice(&seed).unwrap();
        let signature = EcdsaSignature(keypair.sign(msg.as_ref()).0);
        let pubkey = recover_prehashed(eth_hash_message(&msg), &signature).unwrap();
        let address: [u8; 20] =
            sp_core::keccak_256(&pubkey.serialize_uncompressed()[1..])[12..]
                .try_into()
                .unwrap();

        let identity = Identity::Address20(address);
        let id_with_auth = IdentityWithAuth::Ecdsa(identity, signature);
        let index = 1;
    }: _(RawOrigin::Signed(caller.clone()), id_with_auth, index)
    verify {
        assert_eq!(Guild::<T>::user_data(caller, index), Some(identity));
    }
    create_guild {
        let caller: T::AccountId = whitelisted_caller();
        let guild_name = [0u8; 32];
        let n in 0 .. 256;
        let metadata = vec![0u8; n as usize];
    }: _(RawOrigin::Signed(caller), guild_name, metadata)
    verify {
        assert!(Guild::<T>::guild_id(guild_name).is_some());
    }

    impl_benchmark_test_suite!(Guild, crate::mock::new_test_ext(), crate::mock::TestRuntime, extra = false);
}
