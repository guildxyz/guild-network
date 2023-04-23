#![cfg_attr(not(feature = "std"), no_std)]
#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(unused_crate_dependencies)]

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmark;
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
    use gn_common::filter::{Filter, Logic as FilterLogic};
    use gn_common::identity::{Identity, IdentityWithAuth};
    use gn_common::merkle::{Leaf as MerkleLeaf, Proof as MerkleProof};
    use gn_common::{
        Guild, GuildName, Request, RequestData, RequestIdentifier, Role, RoleName, SerializedData,
        SerializedRequirements,
    };
    use pallet_guild_identity::Config as IdentityConfig;
    use pallet_oracle::{Config as OracleConfig, OracleAnswer};
    use sp_std::vec::Vec as SpVec;

    type BalanceOf<T> = <<T as OracleConfig>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::Balance;

    #[pallet::storage]
    #[pallet::getter(fn nonce)]
    pub type Nonce<T: Config> = StorageValue<_, u64, ValueQuery>;

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
    pub type Roles<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, Role, OptionQuery>;

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

    #[pallet::config]
    pub trait Config: OracleConfig<Callback = Call<Self>> + frame_system::Config {
        #[pallet::constant]
        type MaxAllowlistLen: Get<u32>;
        #[pallet::constant]
        type MaxRolesPerGuild: Get<u32>;
        #[pallet::constant]
        type MaxReqsPerRole: Get<u32>;
        #[pallet::constant]
        type MaxSerializedLen: Get<u32>;
        type MyRandomness: Randomness<Self::Hash, Self::BlockNumber>;
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        AllowlistWritten(SpVec<u8>),
        GuildCreated(T::AccountId, GuildName),
        RoleCreated(T::AccountId, GuildName, RoleName),
        RoleAssigned(T::AccountId, GuildName, RoleName),
        RoleStripped(T::AccountId, GuildName, RoleName),
    }

    #[pallet::error]
    pub enum Error<T> {
        AccessDenied,
        GuildAlreadyExists,
        GuildDoesNotExist,
        RoleAlreadyExists,
        RoleDoesNotExist,
        InvalidAllowlistLen,
        InvalidOracleAnswer,
        InvalidOracleRequest,
        UserNotRegistered,
        IdNotRegistered,
        CodecError,
        MaxRolesPerGuildExceeded,
        MaxReqsPerRoleExceeded,
        MaxSerializedLenExceeded,
        MissingAllowlistProof,
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight((<T as Config>::WeightInfo::register(), Pays::No))]
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
        #[pallet::weight((<T as Config>::WeightInfo::join(), Pays::No))]
        pub fn join(
            origin: OriginFor<T>,
            guild_name: GuildName,
            role_name: RoleName,
            proof: Option<MerkleProof>,
        ) -> DispatchResult {
            let signer = ensure_signed(origin.clone())?;
            let role_id = Self::checked_role_id(&signer, &guild_name, &role_name)?;
            // should not throw an error because we already checked that
            // 'role_id' exists
            let role_data = Roles::<T>::get(role_id).ok_or(Error::<T>::RoleDoesNotExist)?;
            // check the onchain filter first
            let (onchain_access, logic) = match role_data.filter {
                Some(Filter::Guild(filter, logic)) => {
                    let access = Self::check_parent_role(&signer, &filter);
                    (access, logic)
                }
                Some(Filter::Allowlist(root, logic, n_leaves)) => {
                    let Some(proof) = proof else {
                        return Err(Error::<T>::MissingAllowlistProof.into())
                    };
                    let id = Self::user_data(&signer, proof.id_index)
                        .ok_or(Error::<T>::IdNotRegistered)?;
                    let leaf = MerkleLeaf::Value(id.as_ref());
                    let access = proof.verify(&root, n_leaves as usize, leaf);
                    (access, logic)
                }
                None => (true, FilterLogic::And),
            };

            match (onchain_access, logic, role_data.requirements.is_some()) {
                // access is granted without the need for an oracle check if
                // T || T
                // T || F
                // T && F
                (true, FilterLogic::Or, _) | (true, _, false) => {
                    Members::<T>::insert(role_id, &signer, true);
                    Self::deposit_event(Event::RoleAssigned(signer, guild_name, role_name));
                    Ok(())
                }
                // access is denied without the need of an oracle check if
                // F && T
                // F && F
                // F || F
                (false, FilterLogic::And, _) | (false, FilterLogic::Or, false) => {
                    Err(Error::<T>::AccessDenied.into())
                }
                // else we need external oracle checks
                // T && T
                // F || T
                (true, FilterLogic::And, true) | (false, FilterLogic::Or, true) => {
                    let data = RequestData::ReqCheck {
                        account: signer.clone(),
                        guild_name,
                        role_name,
                    };
                    let request = Request {
                        requester: signer,
                        data,
                    };
                    let call: <T as OracleConfig>::Callback = Call::callback {
                        result: SpVec::new(),
                    };
                    let fee = BalanceOf::<T>::unique_saturated_from(
                        <T as OracleConfig>::MinimumFee::get(),
                    );

                    if role_data.requirements.is_some() {
                        <pallet_oracle::Pallet<T>>::initiate_request(
                            origin,
                            call,
                            request.encode(),
                            fee,
                        )?;
                    }

                    Ok(())
                }
            }
        }

        #[pallet::call_index(2)]
        #[pallet::weight((<T as Config>::WeightInfo::leave(), Pays::No))]
        pub fn leave(
            origin: OriginFor<T>,
            guild_name: GuildName,
            role_name: RoleName,
        ) -> DispatchResult {
            let signer = ensure_signed(origin.clone())?;
            let role_id = Self::checked_role_id(&signer, &guild_name, &role_name)?;
            Members::<T>::remove(role_id, &signer);
            Self::deposit_event(Event::RoleStripped(signer, guild_name, role_name));
            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::request_oracle_check())]
        pub fn request_oracle_check(
            origin: OriginFor<T>,
            account: T::AccountId,
            guild_name: GuildName,
            role_name: RoleName,
        ) -> DispatchResult {
            let requester = ensure_signed(origin.clone())?;

            let role_id = Self::checked_role_id(&account, &guild_name, &role_name)?;
            // self checking is not allowed, users should just call 'leave'
            ensure!(account != requester, DispatchError::BadOrigin);
            // checked account must be a joined member, otherwise the oracle
            // could unknowingly add the user without checking on-chain filters
            // in the callback
            ensure!(
                Self::member(role_id, &account).is_some(),
                Error::<T>::InvalidOracleRequest
            );
            let role_data = Self::role(role_id).ok_or(Error::<T>::RoleDoesNotExist)?;
            match role_data.filter {
                // NOTE if there is a filter with OR logic
                // then it doesn't make sense to call an oracle check
                // because the filter is definitely satisfied
                Some(Filter::Guild(_, FilterLogic::Or))
                | Some(Filter::Allowlist(_, FilterLogic::Or, _)) => {
                    return Err(Error::<T>::InvalidOracleRequest.into())
                }
                // NOTE it makes sense to perform an oracle check
                // if there's no filter or there's an AND gate
                // between the filter and the requirements
                _ => {}
            }

            if role_data.requirements.is_some() {
                let data = RequestData::ReqCheck {
                    account,
                    guild_name,
                    role_name,
                };
                let request = Request { requester, data };
                let call: <T as OracleConfig>::Callback = Call::callback {
                    result: SpVec::new(),
                };
                let fee =
                    BalanceOf::<T>::unique_saturated_from(<T as OracleConfig>::MinimumFee::get());

                <pallet_oracle::Pallet<T>>::initiate_request(origin, call, request.encode(), fee)?;
                Ok(())
            } else {
                Err(Error::<T>::InvalidOracleRequest.into())
            }
        }

        #[pallet::call_index(4)]
        #[pallet::weight((<T as Config>::WeightInfo::create_guild(metadata.len() as u32), Pays::No))]
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

            ensure!(
                metadata.len() <= T::MaxSerializedLen::get() as usize,
                Error::<T>::MaxSerializedLenExceeded
            );

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

        #[pallet::call_index(5)]
        #[pallet::weight((<T as Config>::WeightInfo::create_free_role(), Pays::No))]
        pub fn create_free_role(
            origin: OriginFor<T>,
            guild_name: GuildName,
            role_name: RoleName,
        ) -> DispatchResult {
            Self::create_role(origin, guild_name, role_name, None, None)?;
            Ok(())
        }

        #[pallet::call_index(6)]
        #[pallet::weight((
            <T as Config>::WeightInfo::create_role_with_allowlist(
                allowlist.len() as u32,
                T::MaxReqsPerRole::get(),
                T::MaxSerializedLen::get()
            ),
            Pays::No
        ))]
        pub fn create_role_with_allowlist(
            origin: OriginFor<T>,
            guild_name: GuildName,
            role_name: RoleName,
            allowlist: SpVec<Identity>,
            filter_logic: gn_common::filter::Logic,
            requirements: Option<SerializedRequirements>,
        ) -> DispatchResult {
            ensure!(
                !allowlist.is_empty() && allowlist.len() <= T::MaxAllowlistLen::get() as usize,
                Error::<T>::InvalidAllowlistLen
            );
            let filter = Filter::allowlist(&allowlist, filter_logic);
            let role_id =
                Self::create_role(origin, guild_name, role_name, Some(filter), requirements)?;

            let offchain_key = gn_common::offchain_allowlist_key(role_id.as_ref());
            sp_io::offchain_index::set(&offchain_key, &allowlist.encode());
            Self::deposit_event(Event::AllowlistWritten(offchain_key));
            Ok(())
        }

        #[pallet::call_index(7)]
        #[pallet::weight((
            <T as Config>::WeightInfo::create_child_role(
                T::MaxReqsPerRole::get(),
                T::MaxSerializedLen::get()
            ),
            Pays::No
        ))]
        pub fn create_child_role(
            origin: OriginFor<T>,
            guild_name: GuildName,
            role_name: RoleName,
            filter: gn_common::filter::Guild,
            filter_logic: gn_common::filter::Logic,
            requirements: Option<SerializedRequirements>,
        ) -> DispatchResult {
            let guild_id = Self::guild_id(filter.name).ok_or(Error::<T>::GuildDoesNotExist)?;
            if let Some(parent_role_name) = filter.role {
                ensure!(
                    RoleIdMap::<T>::contains_key(guild_id, parent_role_name),
                    Error::<T>::RoleDoesNotExist
                );
            }
            let filter = Filter::Guild(filter, filter_logic);
            Self::create_role(origin, guild_name, role_name, Some(filter), requirements)?;
            Ok(())
        }

        #[pallet::call_index(8)]
        #[pallet::weight((
            <T as Config>::WeightInfo::create_unfiltered_role(
                requirements.0.len() as u32,
                T::MaxSerializedLen::get()
            ),
            Pays::No
        ))]
        pub fn create_unfiltered_role(
            origin: OriginFor<T>,
            guild_name: GuildName,
            role_name: RoleName,
            requirements: SerializedRequirements,
        ) -> DispatchResult {
            Self::create_role(origin, guild_name, role_name, None, Some(requirements))?;
            Ok(())
        }

        #[pallet::call_index(9)]
        #[pallet::weight((0, DispatchClass::Operational, Pays::No))]
        pub fn callback(origin: OriginFor<T>, result: SerializedData) -> DispatchResult {
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

        fn create_role(
            origin: OriginFor<T>,
            guild_name: GuildName,
            role_name: RoleName,
            filter: Option<Filter>,
            requirements: Option<SerializedRequirements>,
        ) -> Result<T::Hash, DispatchError> {
            let signer = ensure_signed(origin)?;
            let guild_id = Self::guild_id(guild_name).ok_or(Error::<T>::GuildDoesNotExist)?;
            ensure!(
                !RoleIdMap::<T>::contains_key(guild_id, role_name),
                Error::<T>::RoleAlreadyExists
            );

            if let Some((reqs, logic)) = requirements.as_ref() {
                ensure!(
                    reqs.len() <= T::MaxReqsPerRole::get() as usize,
                    Error::<T>::MaxReqsPerRoleExceeded
                );
                ensure!(
                    logic.len() <= T::MaxSerializedLen::get() as usize,
                    Error::<T>::MaxSerializedLenExceeded
                );
                for req in reqs {
                    ensure!(
                        req.len() <= T::MaxSerializedLen::get() as usize,
                        Error::<T>::MaxSerializedLenExceeded
                    );
                }
            }

            Guilds::<T>::try_mutate(guild_id, |maybe_guild| {
                if let Some(guild) = maybe_guild {
                    if guild.owner != signer {
                        Err(DispatchError::BadOrigin)
                    } else if guild.roles.len() == T::MaxRolesPerGuild::get() as usize {
                        Err(Error::<T>::MaxRolesPerGuildExceeded.into())
                    } else {
                        guild.roles.push(role_name);
                        Ok(())
                    }
                } else {
                    // shouldn't occur because we already
                    // checked that the guild id exists but
                    // better to make sure
                    Err(Error::<T>::GuildDoesNotExist.into())
                }
            })?;

            let role_id = Self::get_random_uuid();
            RoleIdMap::<T>::insert(guild_id, role_name, role_id);
            Roles::<T>::insert(
                role_id,
                Role {
                    filter,
                    requirements,
                },
            );
            Self::deposit_event(Event::RoleCreated(signer, guild_name, role_name));
            Ok(role_id)
        }

        fn check_parent_role(account: &T::AccountId, parent: &gn_common::filter::Guild) -> bool {
            let Some(guild_id) = Self::guild_id(parent.name) else { return false };
            if let Some(parent_role_name) = parent.role {
                let Some(role_id) = Self::role_id(guild_id, parent_role_name) else { return false };
                Self::member(role_id, account).is_some()
            } else {
                let mut access = false;
                // not a very expensive computation if we allow a sensible amount (<30) of roles
                let Some(guild) = Self::guild(guild_id) else { return false };
                for role_name in guild.roles.iter() {
                    let Some(role_id) =
                        Self::role_id(guild_id, role_name) else { return false };
                    if Self::member(role_id, account).is_some() {
                        access = true;
                        break;
                    }
                }
                access
            }
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
