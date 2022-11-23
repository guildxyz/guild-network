#[cfg(test)]
mod test;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::traits::Randomness;
    use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
    use frame_system::{ensure_root, pallet_prelude::*};
    use pallet_chainlink::{CallbackWithParameter, Config as ChainlinkConfig};

    #[pallet::config]
    pub trait Config: ChainlinkConfig<Callback = Call<Self>> + frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type WeightInfo: Sized;
        type MyRandomness: Randomness<Self::Hash, Self::BlockNumber>;
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(1_000_000)]
        pub fn callback(origin: OriginFor<T>, result: Vec<u8>) -> DispatchResult {
            ensure_root(origin)?;
            let res = u64::from_le_bytes(
                result[0..8]
                    .try_into()
                    .map_err(|_| Error::<T>::DecodingFailed)?,
            );
            OracleAnswer::<T>::put(res);
            Ok(())
        }
    }

    #[pallet::storage]
    #[pallet::getter(fn result)]
    pub type OracleAnswer<T: Config> = StorageValue<_, u64, ValueQuery>;

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
            match self {
                Call::callback { .. } => Some(Call::callback { result }),
                _ => None,
            }
        }
    }
}
