#[cfg(test)]
mod test;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
    use frame_system::{ensure_root, pallet_prelude::*};
    use pallet_chainlink::{CallbackWithParameter, Config as ChainlinkConfig};

    #[pallet::config]
    pub trait Config: ChainlinkConfig<Callback = Call<Self>> + frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type WeightInfo: Sized;
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(1_000_000)]
        pub fn callback(origin: OriginFor<T>, result: Vec<u8>) -> DispatchResult {
            ensure_root(origin)?;
            let full_response: u128 =
                u128::decode(&mut &result[..]).map_err(|_| Error::<T>::DecodingFailed)?;
            let res: u64 = (full_response >> 64) as u64;
            Result::<T>::put(res);
            Ok(())
        }
    }

    #[pallet::storage]
    #[pallet::getter(fn result)]
    pub type Result<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::error]
    pub enum Error<T> {
        DecodingFailed,
    }

    #[pallet::event]
    pub enum Event<T: Config> {}

    impl<T: Config> CallbackWithParameter for Call<T> {
        fn with_result(&self, result: Vec<u8>) -> Option<Self> {
            match *self {
                Call::callback { result: _ } => Some(Call::callback { result }),
                _ => None,
            }
        }
    }
}
