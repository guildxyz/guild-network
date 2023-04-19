#![cfg_attr(not(feature = "std"), no_std)]
#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(unused_crate_dependencies)]

pub use pallet::*;

pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    use super::weights::WeightInfo;
    use frame_support::pallet_prelude::*;
    use frame_support::BoundedBTreeMap;
    use frame_system::pallet_prelude::OriginFor;
    use frame_system::{ensure_root, ensure_signed};
    use pallet_oracle::{CallbackWithParameter, Config as OracleConfig, OracleAnswer};

    use sp_std::vec::Vec as SpVec;

    type Prefix = [u8; 8];
    type Identity = [u8; 32];
    type Authority = [u8; 32];

    #[pallet::config]
    pub trait Config: OracleConfig<Callback = Call<Self>> + frame_system::Config {
        #[pallet::constant]
        type MaxLinkedAddresses: Get<u32>;
        #[pallet::constant]
        type MaxLinkedAddressTypes: Get<u32>;
        #[pallet::constant]
        type MaxLinkedIdentityTypes: Get<u32>;
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo;
    }

    #[pallet::storage]
    #[pallet::getter(fn addresses)]
    pub type Addresses<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedBTreeMap<
            Prefix,
            BoundedVec<T::AccountId, T::MaxLinkedAddresses>,
            T::MaxLinkedAddressTypes,
        >,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn identities)]
    pub type Identities<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedBTreeMap<Prefix, Identity, T::MaxLinkedIdentityTypes>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn authorities)]
    pub type Authorities<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, [Option<Authority>; 2], OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Authorized(T::AccountId, Authority),
        AccountRegistered(T::AccountId),
        AccountDeregistered(T::AccountId),
        AddressLinked(T::AccountId, Prefix, T::AccountId),
        AddressRemoved(T::AccountId, Prefix, T::AccountId),
        IdentityLinked(T::AccountId, Prefix, Identity),
        IdentityRemoved(T::AccountId, Prefix, Identity),
    }

    #[pallet::error]
    pub enum Error<T> {
        AccountDoesNotExist,
        AccountAlreadyExists,
        AddressAlreadyLinked,
        AddressDoesNotExist,
        IdentityAlreadyLinked,
        IdentityDoesNotExist,
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight((<T as Config>::WeightInfo::register(), Pays::No))]
        pub fn register(origin: OriginFor<T>) -> DispatchResult {
            let signer = ensure_signed(origin)?;
            ensure!(
                !Authorities::<T>::contains_key(&signer),
                Error::<T>::AccountAlreadyExists
            );
            Authorities::<T>::insert(&signer, [None::<[u8; 32]>; 2]);
            Self::deposit_event(Event::AccountRegistered(signer));
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight((<T as Config>::WeightInfo::deregister(), Pays::No))]
        pub fn deregister(origin: OriginFor<T>) -> DispatchResult {
            let signer = ensure_signed(origin)?;
            ensure!(
                Authorities::<T>::contains_key(&signer),
                Error::<T>::AccountDoesNotExist
            );
            Addresses::<T>::remove(&signer);
            Identities::<T>::remove(&signer);
            Authorities::<T>::remove(&signer);
            Self::deposit_event(Event::AccountDeregistered(signer));
            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight((<T as Config>::WeightInfo::authorize(), Pays::No))]
        pub fn authorize(origin: OriginFor<T>, authority: Authority) -> DispatchResult {
            let signer = ensure_signed(origin)?;
            Authorities::<T>::try_mutate(signer, |maybe_authorities| {
                if let Some(authorities) = maybe_authorities {
                    match authorities {
                        [Some(_), None] => authorities[1] = Some(authority),
                        _ => authorities[0] = Some(authority),
                    }
                    Ok(())
                } else {
                    Err(Error::<T>::AccountDoesNotExist)
                }
            })?;
            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight((<T as Config>::WeightInfo::link_address(), Pays::No))]
        pub fn link_address(
            origin: OriginFor<T>,
            prefix: Prefix,
            address: T::AccountId,
        ) -> DispatchResult {
            todo!()
        }

        #[pallet::call_index(4)]
        #[pallet::weight((<T as Config>::WeightInfo::unlink_address(), Pays::No))]
        pub fn unlink_address(origin: OriginFor<T>, prefix: Prefix) -> DispatchResult {
            todo!()
        }

        #[pallet::call_index(5)]
        #[pallet::weight((<T as Config>::WeightInfo::link_identity(), Pays::No))]
        pub fn link_identity(
            origin: OriginFor<T>,
            prefix: Prefix,
            address: Identity,
        ) -> DispatchResult {
            todo!()
        }

        #[pallet::call_index(6)]
        #[pallet::weight((<T as Config>::WeightInfo::unlink_identity(), Pays::No))]
        pub fn unlink_identity(origin: OriginFor<T>, prefix: Prefix) -> DispatchResult {
            todo!()
        }

        #[pallet::call_index(9)]
        #[pallet::weight((0, DispatchClass::Operational, Pays::No))]
        pub fn callback(origin: OriginFor<T>, result: SpVec<u8>) -> DispatchResult {
            ensure_root(origin)?;
            todo!();
        }
    }
}
