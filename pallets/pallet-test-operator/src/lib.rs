pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
    use frame_system::{ensure_root, pallet_prelude::*};

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
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
}

impl<T: pallet::Config> pallet_chainlink::CallbackWithParameter for pallet::Call<T> {
    fn with_result(&self, result: Vec<u8>) -> Option<Self> {
        match *self {
            pallet::Call::callback { result: _ } => Some(pallet::Call::callback { result }),
            _ => None,
        }
    }
}
