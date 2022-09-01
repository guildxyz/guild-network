#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use codec::{Decode, Encode};
    use frame_support::{
        dispatch::DispatchResult, pallet_prelude::*, sp_runtime::traits::UniqueSaturatedFrom,
        traits::Currency,
    };
    use frame_system::{ensure_root, pallet_prelude::*};
    use sp_std::prelude::*;

    use pallet_chainlink::{CallbackWithParameter, Config as ChainlinkTrait};

    type BalanceOf<T> = <<T as pallet_chainlink::Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::Balance;

    #[pallet::config]
    pub trait Config: ChainlinkTrait + frame_system::Config {
        type Event: From<Event<Self>>
            + Into<<Self as frame_system::Config>::Event>
            + IsType<<Self as frame_system::Config>::Event>;
        type Callback: From<Call<Self>> + Into<<Self as ChainlinkTrait>::Callback>;
    }

    #[pallet::storage]
    #[pallet::getter(fn result)]
    pub(super) type Result<T: Config> = StorageValue<_, u128, ValueQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000_000)]
        pub fn send_request(
            origin: OriginFor<T>,
            operator: T::AccountId,
            specid: Vec<u8>,
        ) -> DispatchResult {
            let parameters = "something";
            let call: <T as Config>::Callback = Call::callback { result: vec![] }.into();

            let fee = BalanceOf::<T>::unique_saturated_from(100_000_000u32);

            <pallet_chainlink::Pallet<T>>::initiate_request(
                origin,
                operator,
                specid.clone(),
                0,
                parameters.encode(),
                fee,
                call.into(),
            )?;

            Self::deposit_event(Event::RequestSent(specid, parameters.encode()));

            Ok(())
        }

        #[pallet::weight(0)]
        pub fn callback(origin: OriginFor<T>, result: Vec<u8>) -> DispatchResult {
            ensure_root(origin)?;

            // The result is expected to be a SCALE encoded `u128`
            let r: u128 = u128::decode(&mut &result[..]).map_err(|_| Error::<T>::DecodingFailed)?;

            Self::deposit_event(Event::ExampleCallback(result, r));

            Result::<T>::put(r);

            Ok(())
        }
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::error]
    pub enum Error<T> {
        DecodingFailed,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        RequestSent(Vec<u8>, Vec<u8>),
        ExampleCallback(Vec<u8>, u128),
    }

    impl<T: Config> CallbackWithParameter for Call<T> {
        fn with_result(&self, result: Vec<u8>) -> Option<Self> {
            match *self {
                Call::callback { result: _ } => Some(Call::callback { result }),
                _ => None,
            }
        }
    }
}
