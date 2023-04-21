#![cfg_attr(not(feature = "std"), no_std)]
#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(unused_crate_dependencies)]

pub use pallet::*;

pub mod weights;

#[allow(clippy::type_complexity)]
#[frame_support::pallet]
pub mod pallet {
    use super::weights::WeightInfo;
    use frame_support::pallet_prelude::*;
    use frame_support::{
        sp_runtime::traits::UniqueSaturatedFrom, traits::Currency, BoundedBTreeMap,
    };
    use frame_system::pallet_prelude::OriginFor;
    use frame_system::{ensure_root, ensure_signed};
    use gn_common::{Authority, Identity, LinkIdentityRequest, Prefix, SerializedData};
    use pallet_oracle::{CallbackWithParameter, Config as OracleConfig, OracleAnswer};

    type BalanceOf<T> = <<T as OracleConfig>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::Balance;

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
        AddressUnlinked(T::AccountId, Prefix, T::AccountId),
        IdentityLinked(T::AccountId, Prefix, Identity),
        IdentityUnlinked(T::AccountId, Prefix, Identity),
    }

    #[pallet::error]
    pub enum Error<T> {
        AccountDoesNotExist,
        AccountAlreadyExists,
        AddressAlreadyLinked,
        AddressDoesNotExist,
        AddressPrefixDoesNotExist,
        IdentityAlreadyLinked,
        IdentityDoesNotExist,
        IdentityCheckFailed,
        InvalidAuthoritySignature,
        InvalidOracleAnswer,
        MaxLinkedAddressesExceeded,
        MaxLinkedAddressTypesExceeded,
        MaxLinkedIdentitiesExceeded,
        UnknownAuthority,
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
            Addresses::<T>::insert(&signer, BoundedBTreeMap::new());
            Identities::<T>::insert(&signer, BoundedBTreeMap::new());
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
            Authorities::<T>::try_mutate(&signer, |maybe_authorities| {
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
            Self::deposit_event(Event::Authorized(signer, authority));
            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight((<T as Config>::WeightInfo::link_address(), Pays::No))]
        pub fn link_address(
            origin: OriginFor<T>,
            primary: T::AccountId,
            prefix: Prefix,
            auth_sig: [u8; 65],
        ) -> DispatchResult {
            let signer = ensure_signed(origin)?;
            // verify authority signature
            let message = gn_sig::webcrypto::hash_account_id(&signer);
            let authority_pubkey = gn_sig::webcrypto::recover_prehashed(&message, &auth_sig)
                .ok_or(Error::<T>::InvalidAuthoritySignature)?;
            let hashed_authority_pubkey = gn_sig::webcrypto::hash_pubkey(&authority_pubkey);
            let authorities =
                Authorities::<T>::get(&signer).ok_or(Error::<T>::AccountDoesNotExist)?;
            if !authorities
                .iter()
                .any(|authority| authority == &Some(hashed_authority_pubkey))
            {
                return Err(Error::<T>::UnknownAuthority.into());
            }

            Addresses::<T>::try_mutate(&primary, |maybe_address_map| {
                if let Some(address_map) = maybe_address_map {
                    if let Some(address_vec) = address_map.get_mut(&prefix) {
                        if address_vec.iter().any(|address| address == &signer) {
                            return Err(Error::<T>::AddressAlreadyLinked);
                        }
                        address_vec
                            .try_push(signer.clone())
                            .map_err(|_| Error::<T>::MaxLinkedAddressesExceeded)?;
                    } else {
                        let mut address_vec = BoundedVec::with_max_capacity();
                        // should never fail because we just created a new
                        // vector however, handling the error shouldn't hurt.
                        // It would only fail if the bound for the vec is 0 and
                        // it's questionable whether that would ever occur
                        address_vec
                            .try_push(signer.clone())
                            .map_err(|_| Error::<T>::MaxLinkedAddressesExceeded)?;
                        address_map
                            .try_insert(prefix, address_vec)
                            .map_err(|_| Error::<T>::MaxLinkedAddressTypesExceeded)?;
                    }
                    Ok(())
                } else {
                    Err(Error::<T>::AccountDoesNotExist)
                }
            })?;
            Self::deposit_event(Event::AddressLinked(primary, prefix, signer));
            Ok(())
        }

        #[pallet::call_index(4)]
        #[pallet::weight((<T as Config>::WeightInfo::unlink_address(), Pays::No))]
        pub fn unlink_address(
            origin: OriginFor<T>,
            prefix: Prefix,
            address_to_unlink: T::AccountId,
        ) -> DispatchResult {
            let signer = ensure_signed(origin)?;
            Addresses::<T>::try_mutate(&signer, |maybe_address_map| {
                if let Some(address_map) = maybe_address_map {
                    if let Some(address_vec) = address_map.get_mut(&prefix) {
                        let index = address_vec
                            .iter()
                            .position(|address| address == &address_to_unlink)
                            .ok_or(Error::<T>::AddressDoesNotExist)?;
                        address_vec.remove(index);
                        Ok(())
                    } else {
                        Err(Error::<T>::AddressPrefixDoesNotExist)
                    }
                } else {
                    Err(Error::<T>::AccountDoesNotExist)
                }
            })?;
            Self::deposit_event(Event::AddressUnlinked(signer, prefix, address_to_unlink));
            Ok(())
        }

        #[pallet::call_index(5)]
        #[pallet::weight((<T as Config>::WeightInfo::link_identity(), Pays::No))]
        pub fn link_identity(
            origin: OriginFor<T>,
            prefix: Prefix,
            identity: Identity,
        ) -> DispatchResult {
            let signer = ensure_signed(origin.clone())?;
            let identity_map = Self::identities(&signer).ok_or(Error::<T>::AccountDoesNotExist)?;
            if identity_map.contains_key(&prefix) {
                return Err(Error::<T>::IdentityAlreadyLinked.into());
            }
            // compile and send oracle request
            let call: <T as OracleConfig>::Callback = Call::callback {
                result: SerializedData::new(),
            };
            let fee = BalanceOf::<T>::unique_saturated_from(<T as OracleConfig>::MinimumFee::get());
            let request = LinkIdentityRequest {
                requester: signer,
                prefix,
                identity,
            };
            <pallet_oracle::Pallet<T>>::initiate_request(origin, call, request.encode(), fee)?;
            Ok(())
        }

        #[pallet::call_index(6)]
        #[pallet::weight((<T as Config>::WeightInfo::unlink_identity(), Pays::No))]
        pub fn unlink_identity(origin: OriginFor<T>, prefix: Prefix) -> DispatchResult {
            let signer = ensure_signed(origin)?;
            Identities::<T>::try_mutate(&signer, |maybe_identity_map| {
                if let Some(identity_map) = maybe_identity_map {
                    let identity = identity_map
                        .remove(&prefix)
                        .ok_or(Error::<T>::IdentityDoesNotExist)?;
                    Self::deposit_event(Event::IdentityUnlinked(signer.clone(), prefix, identity));
                    Ok(())
                } else {
                    Err(Error::<T>::AccountDoesNotExist)
                }
            })?;
            Ok(())
        }

        #[pallet::call_index(9)]
        #[pallet::weight((0, DispatchClass::Operational, Pays::No))]
        pub fn callback(origin: OriginFor<T>, result: SerializedData) -> DispatchResult {
            ensure_root(origin)?;
            // cannot wrap codec::Error in this error type because
            // it doesn't implement the required traits
            let answer = OracleAnswer::decode(&mut result.as_slice())
                .map_err(|_| Error::<T>::InvalidOracleAnswer)?;

            ensure!(answer.result.len() == 1, Error::<T>::InvalidOracleAnswer);

            let request = LinkIdentityRequest::<T::AccountId>::decode(&mut answer.data.as_slice())
                .map_err(|_| Error::<T>::InvalidOracleAnswer)?;

            if answer.result[0] == 1 {
                return Err(Error::<T>::IdentityCheckFailed.into());
            }

            Identities::<T>::try_mutate(request.requester.clone(), |maybe_identity_map| {
                if let Some(identity_map) = maybe_identity_map {
                    if !identity_map.contains_key(&request.prefix) {
                        identity_map
                            .try_insert(request.prefix, request.identity)
                            .map_err(|_| Error::<T>::MaxLinkedIdentitiesExceeded)?;
                        Self::deposit_event(Event::IdentityLinked(
                            request.requester,
                            request.prefix,
                            request.identity,
                        ));
                        Ok(())
                    } else {
                        Err(Error::<T>::IdentityAlreadyLinked)
                    }
                } else {
                    Err(Error::<T>::AccountDoesNotExist)
                }
            })?;

            todo!();
        }
    }

    impl<T: Config> CallbackWithParameter for Call<T> {
        fn with_result(&self, result: SerializedData) -> Option<Self> {
            match self {
                Call::callback { .. } => Some(Call::callback { result }),
                _ => None,
            }
        }
    }
}
