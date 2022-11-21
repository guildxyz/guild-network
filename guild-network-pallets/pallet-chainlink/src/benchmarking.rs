use super::*;
use crate::Pallet as Chainlink;

use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

benchmarks! {
    register_operator {
        let caller: T::AccountId = whitelisted_caller();
    }: _(RawOrigin::Signed(caller))
    verify {
    }
    deregister_operator {
        let operator: T::AccountId = whitelisted_caller();
        Chainlink::<T>::register_operator(RawOrigin::Signed(operator.clone()).into())?;
    }: _(RawOrigin::Signed(operator))
    verify {
    }
    // TODO it's hard to benchmark these as they are taking dynamic inputs
    // and they are coupled with other pallets
    //initiate_request {
    //    let caller: T::AccountId = whitelisted_caller();
    //    let operator: T::AccountId = account("operator", 1, 123);

    //    Chainlink::<T>::register_operator(RawOrigin::Signed(operator.clone()).into())?;

    //    let spec_index = vec![0; 5];
    //    let data_version = 987_u64;
    //    let data = ["this", "and", "that"].encode();
    //    let fee = T::Currency::minimum_balance();
    //}: _(RawOrigin::Signed(caller), operator, spec_index, data_version, data, fee,
    //verify {
    //}
    //callback {
    //    let b in 0..1000;

    //    let caller: T::AccountId = whitelisted_caller();
    //    let request_id = 128_u64;
    //    let operator: T::AccountId = account("operator", 1, 123);

    //    Chainlink::<T>::register_operator(RawOrigin::Signed(operator.clone()).into())?;
    //    // TODO should add an initiate request

    //}: _(RawOrigin::Signed(caller), request_id, vec![0; b as usize])
    //verify {
    //}
}
