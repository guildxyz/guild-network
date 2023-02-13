#![cfg_attr(not(feature = "std"), no_std)]
#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(unused_crate_dependencies)]

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod test;
pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    use super::weights::WeightInfo;
    use frame_support::traits::Randomness;
    use frame_support::{
        dispatch::DispatchResult, pallet_prelude::*, sp_runtime::traits::UniqueSaturatedFrom,
        traits::Currency, StorageDoubleMap as StorageDoubleMapT,
    };
    use frame_system::pallet_prelude::*;
    use gn_common::identity::{Identity, IdentityWithAuth};
    use gn_common::{Guild, GuildName, Request, RequestData, RequestIdentifier, Role, RoleName};
    use gn_requirement::filter::{Guild as GuildFilter, Logic as FilterLogic};
    use pallet_oracle::{CallbackWithParameter, Config as OracleConfig, OracleAnswer};
    use sp_std::vec::Vec as SpVec;

    type BalanceOf<T> = <<T as OracleConfig>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::Balance;

    #[pallet::storage]
    #[pallet::getter(fn nonce)]
    pub type Nonce<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn next_request_id)]
    pub type NextRequestIdentifier<T: Config> = StorageValue<_, RequestIdentifier, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn guild_id)]
    pub type GuildIdMap<T: Config> =
        StorageMap<_, Blake2_128Concat, GuildName, T::Hash, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn role_id)]
    pub type RoleIdMap<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::Hash, // Guild id
        Blake2_128Concat,
        RoleName, // Role name
        T::Hash,  // Role id
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn guild)]
    pub type Guilds<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, Guild<T::AccountId>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn role)]
    pub type Roles<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, RoleData, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn member)]
    pub type Members<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::Hash,
        Blake2_128Concat,
        T::AccountId,
        bool,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn user_data)]
    pub type UserData<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        u8,
        Identity,
        OptionQuery,
    >;

    #[pallet::config]
    pub trait Config: OracleConfig<Callback = Call<Self>> + frame_system::Config {
        #[pallet::constant]
        type MaxRolesPerGuild: Get<u32>;
        #[pallet::constant]
        type MaxReqsPerRole: Get<u32>;
        #[pallet::constant]
        type MaxSerializedReqLen: Get<u32>;
        #[pallet::constant]
        type MaxIdentities: Get<u8>;
        type MyRandomness: Randomness<Self::Hash, Self::BlockNumber>;
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        GuildCreated(T::AccountId, GuildName),
        IdRegistered(T::AccountId, u8),
        RoleAssigned(T::AccountId, GuildName, RoleName),
        RoleStripped(T::AccountId, GuildName, RoleName),
    }

    #[pallet::error]
    pub enum Error<T> {
        AccessDenied,
        GuildAlreadyExists,
        GuildDoesNotExist,
        RoleDoesNotExist,
        InvalidOracleAnswer,
        UserNotRegistered,
        CodecError,
        MaxIdentitiesExceeded,
        MaxRolesPerGuildExceeded,
        MaxReqsPerRoleExceeded,
        MaxSerializedReqLenExceeded,
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(1000)] //T::WeightInfo::register())]
        pub fn register(
            origin: OriginFor<T>,
            identity_with_auth: IdentityWithAuth,
            index: u8,
        ) -> DispatchResult {
            let signer = ensure_signed(origin.clone())?;

            ensure!(
                index < T::MaxIdentities::get(),
                Error::<T>::MaxIdentitiesExceeded
            );

            match identity_with_auth {
                IdentityWithAuth::Other(Identity::Other(_), _) => {
                    let data = RequestData::Register {
                        identity_with_auth,
                        index,
                    };
                    let request = Request::<T::AccountId> {
                        requester: signer,
                        data,
                    };
                    let call: <T as OracleConfig>::Callback = Call::callback {
                        result: SpVec::new(),
                    };
                    let fee = BalanceOf::<T>::unique_saturated_from(
                        <T as OracleConfig>::MinimumFee::get(),
                    );
                    <pallet_oracle::Pallet<T>>::initiate_request(
                        origin,
                        call,
                        request.encode(),
                        fee,
                    )?;
                }
                id_with_auth => {
                    let msg = gn_common::utils::verification_msg(&signer);
                    if id_with_auth.verify(msg) {
                        UserData::<T>::insert(&signer, index, Identity::from(identity_with_auth));
                        Self::deposit_event(Event::IdRegistered(signer, index));
                    } else {
                        return Err(Error::<T>::AccessDenied.into());
                    }
                }
            }

            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(10000000)]
        pub fn request_check(
            origin: OriginFor<T>,
            account: T::AccountId,
            guild_name: GuildName,
            role_name: RoleName,
        ) -> DispatchResult {
            let requester = ensure_signed(origin.clone())?;

            let role_id = Self::checked_role_id(&account, &guild_name, &role_name)?;
            // if account == signer then the user either wants to join or leave
            match (
                account == requester,
                Members::<T>::contains_key(role_id, &account),
            ) {
                (true, true) => {
                    // user wants to be stripped of role
                    Members::<T>::remove(role_id, &account);
                    Self::deposit_event(Event::RoleStripped(account, guild_name, role_name));
                    return Ok(());
                }
                // invalid account in request data (you cannot request
                // other accounts to get assigned a role)
                (false, false) => return Err(DispatchError::BadOrigin),
                // (false, true) keeper wants to request a check
                // (true, false) user wants to get a role assigned
                _ => {}
            }
            let data = RequestData::ReqCheck {
                account,
                guild_name,
                role_name,
            };
            let request = Request { requester, data };
            let call: <T as OracleConfig>::Callback = Call::callback {
                result: SpVec::new(),
            };
            let fee = BalanceOf::<T>::unique_saturated_from(<T as OracleConfig>::MinimumFee::get());
            <pallet_oracle::Pallet<T>>::initiate_request(origin, call, request.encode(), fee)?;

            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(10000000)]
        pub fn create_guild(
            origin: OriginFor<T>,
            guild_name: GuildName,
            metadata: SerializedData,
        ) -> DispatchResult {
            let signer = ensure_signed(origin)?;
            ensure!(
                !GuildIdMap::<T>::contains_key(guild_name),
                Error::<T>::GuildAlreadyExists
            );

            // TODO
            //ensure!(
            //    metadata.length() < T::MaxMetadataLength::get() as usize,
            //    Error::<T>::GuildAlreadyExists
            //);

            let guild_id = Self::get_random_uuid();
            GuildIdMap::<T>::insert(guild_name, guild_id);

            let guild = Guild {
                name: guild_name,
                owner: signer.clone(),
                metadata,
                roles: SpVec::new(),
            };

            Guilds::<T>::insert(guild_id, guild);

            Self::deposit_event(Event::GuildCreated(signer, guild_name));
            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(10000000)]
        pub fn add_role(
            origin: OriginFor<T>,
            guild_name: GuildName,
            role_name: RoleName,
        ) -> DispatchResult {
            todo!()
        }

        #[pallet::call_index(4)]
        #[pallet::weight(0)]
        pub fn callback(origin: OriginFor<T>, result: SerializedData) -> DispatchResult {
            // NOTE this ensures that only the root can call this function via
            // a callback, see `frame_system::RawOrigin`
            ensure_root(origin)?;

            // cannot wrap codec::Error in this error type because
            // it doesn't implement the required traits
            let answer =
                OracleAnswer::decode(&mut result.as_slice()).map_err(|_| Error::<T>::CodecError)?;

            ensure!(answer.result.len() == 1, Error::<T>::InvalidOracleAnswer);

            let access = answer.result[0] == 1;

            let request = Request::<T::AccountId>::decode(&mut answer.data.as_slice())
                .map_err(|_| Error::<T>::CodecError)?;

            match request.data {
                RequestData::ReqCheck {
                    account,
                    guild_name,
                    role_name,
                } => {
                    let role_id = Self::checked_role_id(&account, &guild_name, &role_name)?;
                    match (access, Members::<T>::contains_key(role_id, &account)) {
                        (true, false) => {
                            Members::<T>::insert(role_id, &account, true);
                            Self::deposit_event(Event::RoleAssigned(
                                account, guild_name, role_name,
                            ));
                        }
                        (false, true) => {
                            // TODO send locked rewards to requester
                            Members::<T>::remove(role_id, &account);
                            Self::deposit_event(Event::RoleStripped(
                                account, guild_name, role_name,
                            ));
                        }
                        (false, false) => return Err(Error::<T>::AccessDenied.into()),
                        (true, true) => {} // nothing happens, requirements are still satisfied
                    }
                }
                RequestData::Register {
                    identity_with_auth,
                    index,
                } => {
                    ensure!(access, Error::<T>::AccessDenied);
                    ensure!(
                        index < T::MaxIdentities::get(),
                        Error::<T>::MaxIdentitiesExceeded
                    );
                    UserData::<T>::insert(
                        &request.requester,
                        index,
                        Identity::from(identity_with_auth),
                    );
                    Self::deposit_event(Event::IdRegistered(request.requester, index));
                }
            }

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn checked_role_id(
            account: &T::AccountId,
            guild_name: &GuildName,
            role_name: &RoleName,
        ) -> Result<T::Hash, DispatchError> {
            let guild_id = Self::guild_id(guild_name).ok_or(Error::<T>::GuildDoesNotExist)?;
            let role_id = Self::role_id(guild_id, role_name).ok_or(Error::<T>::RoleDoesNotExist)?;

            // check the requester is registered
            ensure!(
                <UserData<T>>::contains_prefix(account),
                Error::<T>::UserNotRegistered
            );

            Ok(role_id)
        }

        fn get_and_increment_nonce() -> SerializedData {
            let nonce = Nonce::<T>::get();
            Nonce::<T>::put(nonce.wrapping_add(1));
            nonce.encode()
        }

        fn get_random_uuid() -> T::Hash {
            let nonce = Self::get_and_increment_nonce();
            let (random_value, _) = T::MyRandomness::random(&nonce);
            random_value
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
