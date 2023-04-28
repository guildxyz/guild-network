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
        dispatch::DispatchResult,
        pallet_prelude::*,
        sp_runtime::traits::UniqueSaturatedFrom,
        traits::{Currency, PalletInfo},
    };
    use frame_system::pallet_prelude::*;
    use frame_system::RawOrigin;
    use gn_common::filter::{Filter, Logic as FilterLogic};
    use gn_common::merkle::{Leaf as MerkleLeaf, Proof as MerkleProof};
    use gn_common::*;
    use pallet_guild_identity::Config as IdentityConfig;
    use pallet_guild_identity::Pallet as IdentityPallet;
    use pallet_oracle::Config as OracleConfig;
    use sp_std::vec::Vec as SpVec;

    #[pallet::config]
    pub trait Config: IdentityConfig + OracleConfig + frame_system::Config {
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
        NoPalletIndex,
        IdNotRegistered,
        InvalidAllowlistLen,
        InvalidJoinRequest,
        InvalidOracleAnswer,
        InvalidOracleRequest,
        UserNotRegistered,
        UserNotJoined,
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
        #[pallet::weight((<T as Config>::WeightInfo::join_free_role(), Pays::No))]
        pub fn join_free_role(
            origin: OriginFor<T>,
            guild_name: GuildName,
            role_name: RoleName,
        ) -> DispatchResult {
            let signer = ensure_signed(origin.clone())?;
            let (role_id, role_data) = Self::checked_role(&signer, &guild_name, &role_name)?;
            if role_data.filter.is_some() || role_data.requirements.is_some() {
                return Err(Error::<T>::InvalidJoinRequest.into());
            }

            Members::<T>::insert(role_id, &signer, true);
            Self::deposit_event(Event::RoleAssigned(signer, guild_name, role_name));
            Ok(())
        }
        #[pallet::call_index(1)]
        #[pallet::weight((<T as Config>::WeightInfo::join_child_role(), Pays::No))]
        pub fn join_child_role(
            origin: OriginFor<T>,
            guild_name: GuildName,
            role_name: RoleName,
        ) -> DispatchResult {
            let signer = ensure_signed(origin.clone())?;
            let (role_id, role_data) = Self::checked_role(&signer, &guild_name, &role_name)?;
            match (role_data.filter, role_data.requirements) {
                (Some(Filter::Guild(filter, logic)), maybe_requirements) => {
                    let filter_access = Self::check_parent_role(&signer, &filter);
                    Self::evaluate_access(
                        signer,
                        filter_access,
                        logic,
                        maybe_requirements.is_some(),
                        guild_name,
                        role_name,
                        role_id,
                    )
                }
                _ => Err(Error::<T>::InvalidJoinRequest.into()),
            }
        }
        /// Only allows users to join with their primary address.
        ///
        /// Otherwise, if we allow users to join with one of their linked
        /// addresses, then it's problematic to check access if the user
        /// unlinks their address in the meantime.
        #[pallet::call_index(2)]
        #[pallet::weight((<T as Config>::WeightInfo::join_role_with_allowlist(), Pays::No))]
        pub fn join_role_with_allowlist(
            origin: OriginFor<T>,
            guild_name: GuildName,
            role_name: RoleName,
            merkle_proof: MerkleProof,
        ) -> DispatchResult {
            let signer = ensure_signed(origin.clone())?;
            let (role_id, role_data) = Self::checked_role(&signer, &guild_name, &role_name)?;
            match (role_data.filter, role_data.requirements) {
                (Some(Filter::Allowlist(root, logic, n_leaves)), maybe_requirements) => {
                    let encoded_id = signer.encode();
                    let leaf = MerkleLeaf::Value(&encoded_id);
                    let filter_access = merkle_proof.verify(&root, n_leaves as usize, leaf);
                    Self::evaluate_access(
                        signer,
                        filter_access,
                        logic,
                        maybe_requirements.is_some(),
                        guild_name,
                        role_name,
                        role_id,
                    )
                }
                _ => Err(Error::<T>::InvalidJoinRequest.into()),
            }
        }
        #[pallet::call_index(3)]
        #[pallet::weight((<T as Config>::WeightInfo::join_unfiltered_role(), Pays::No))]
        pub fn join_unfiltered_role(
            origin: OriginFor<T>,
            guild_name: GuildName,
            role_name: RoleName,
        ) -> DispatchResult {
            let signer = ensure_signed(origin.clone())?;
            let (_, role_data) = Self::checked_role(&signer, &guild_name, &role_name)?;
            match (role_data.filter, role_data.requirements) {
                (None, Some(_)) => Self::dispatch_oracle_request(signer, guild_name, role_name),
                _ => Err(Error::<T>::InvalidJoinRequest.into()),
            }
        }

        #[pallet::call_index(4)]
        #[pallet::weight((<T as Config>::WeightInfo::leave(), Pays::No))]
        pub fn leave(
            origin: OriginFor<T>,
            guild_name: GuildName,
            role_name: RoleName,
        ) -> DispatchResult {
            let signer = ensure_signed(origin)?;
            let (role_id, _) = Self::checked_role(&signer, &guild_name, &role_name)?;
            Members::<T>::remove(role_id, &signer);
            Self::deposit_event(Event::RoleStripped(signer, guild_name, role_name));
            Ok(())
        }

        #[pallet::call_index(5)]
        #[pallet::weight(<T as Config>::WeightInfo::request_access_check())]
        pub fn request_access_check(
            origin: OriginFor<T>,
            account: T::AccountId,
            guild_name: GuildName,
            role_name: RoleName,
        ) -> DispatchResult {
            let signer = ensure_signed(origin.clone())?;
            // self-checking is not allowed, users should just call 'leave'
            ensure!(account != signer, DispatchError::BadOrigin);
            let guild_id = Self::guild_id(guild_name).ok_or(Error::<T>::GuildDoesNotExist)?;
            let role_id = Self::role_id(guild_id, role_name).ok_or(Error::<T>::RoleDoesNotExist)?;
            if IdentityPallet::<T>::addresses(&account).is_none() {
                Members::<T>::take(role_id, &account).ok_or(Error::<T>::UserNotJoined)?;
                Self::deposit_event(Event::RoleStripped(account, guild_name, role_name));
                return Ok(());
            }
            let role_data = Self::role(role_id).ok_or(Error::<T>::RoleDoesNotExist)?;

            match role_data.filter {
                // NOTE if there is a filter with OR logic
                // then it doesn't make sense to call an oracle check
                // because the filter is definitely satisfied
                Some(Filter::Guild(_, FilterLogic::Or))
                | Some(Filter::Allowlist(_, FilterLogic::Or, _)) => {
                    Err(Error::<T>::InvalidOracleRequest.into())
                }
                // NOTE it makes sense to perform an oracle check
                // if there's no filter or there's an AND gate
                // between the filter and the requirements
                _ => {
                    if role_data.requirements.is_some() {
                        Self::dispatch_oracle_request(account, guild_name, role_name)
                    } else {
                        Err(Error::<T>::InvalidOracleRequest.into())
                    }
                }
            }
        }

        #[pallet::call_index(6)]
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

        #[pallet::call_index(7)]
        #[pallet::weight((<T as Config>::WeightInfo::create_free_role(), Pays::No))]
        pub fn create_free_role(
            origin: OriginFor<T>,
            guild_name: GuildName,
            role_name: RoleName,
        ) -> DispatchResult {
            let signer = ensure_signed(origin)?;
            Self::do_create_role(signer, guild_name, role_name, None, None)?;
            Ok(())
        }

        #[pallet::call_index(8)]
        #[pallet::weight((
            <T as Config>::WeightInfo::create_role_with_allowlist(
                allowlist.len() as u32,
                <T as Config>::MaxReqsPerRole::get(),
                <T as Config>::MaxSerializedLen::get(),
            ),
            Pays::No
        ))]
        pub fn create_role_with_allowlist(
            origin: OriginFor<T>,
            guild_name: GuildName,
            role_name: RoleName,
            allowlist: SpVec<T::AccountId>,
            filter_logic: gn_common::filter::Logic,
            requirements: Option<SerializedRequirements>,
        ) -> DispatchResult {
            let signer = ensure_signed(origin)?;
            ensure!(
                !allowlist.is_empty() && allowlist.len() <= T::MaxAllowlistLen::get() as usize,
                Error::<T>::InvalidAllowlistLen
            );
            let filter = Filter::allowlist(&allowlist, filter_logic);
            let role_id =
                Self::do_create_role(signer, guild_name, role_name, Some(filter), requirements)?;

            let offchain_key = gn_common::offchain_allowlist_key(role_id.as_ref());
            sp_io::offchain_index::set(&offchain_key, &allowlist.encode());
            Self::deposit_event(Event::AllowlistWritten(offchain_key));
            Ok(())
        }

        #[pallet::call_index(9)]
        #[pallet::weight((
            <T as Config>::WeightInfo::create_child_role(
                <T as Config>::MaxReqsPerRole::get(),
                <T as Config>::MaxSerializedLen::get(),
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
            let signer = ensure_signed(origin)?;
            let guild_id = Self::guild_id(filter.name).ok_or(Error::<T>::GuildDoesNotExist)?;
            if let Some(parent_role_name) = filter.role {
                ensure!(
                    RoleIdMap::<T>::contains_key(guild_id, parent_role_name),
                    Error::<T>::RoleDoesNotExist
                );
            }
            let filter = Filter::Guild(filter, filter_logic);
            Self::do_create_role(signer, guild_name, role_name, Some(filter), requirements)?;
            Ok(())
        }

        #[pallet::call_index(10)]
        #[pallet::weight((
            <T as Config>::WeightInfo::create_unfiltered_role(
                <T as Config>::MaxReqsPerRole::get(),
                <T as Config>::MaxSerializedLen::get(),
            ),
            Pays::No
        ))]
        pub fn create_unfiltered_role(
            origin: OriginFor<T>,
            guild_name: GuildName,
            role_name: RoleName,
            requirements: SerializedRequirements,
        ) -> DispatchResult {
            let signer = ensure_signed(origin)?;
            Self::do_create_role(signer, guild_name, role_name, None, Some(requirements))?;
            Ok(())
        }

        #[pallet::call_index(11)]
        #[pallet::weight((0, DispatchClass::Operational))]
        pub fn callback(
            origin: OriginFor<T>,
            request_id: RequestIdentifier,
            result: bool,
        ) -> DispatchResult {
            let signer = ensure_signed(origin)?;
            let request = <pallet_oracle::Pallet<T>>::request(request_id)
                .ok_or(Error::<T>::InvalidOracleAnswer)?;

            let pallet_index = <T as frame_system::Config>::PalletInfo::index::<Self>()
                .ok_or(Error::<T>::NoPalletIndex)?;

            ensure!(
                request.pallet_index == pallet_index as u32,
                Error::<T>::InvalidOracleAnswer
            );

            let request = AccessCheckRequest::<T::AccountId>::decode(&mut request.data.as_slice())
                .map_err(|_| Error::<T>::InvalidOracleAnswer)?;

            <pallet_oracle::Pallet<T>>::callback(RawOrigin::Root, signer, request_id)?;

            let guild_id =
                Self::guild_id(request.guild_name).ok_or(Error::<T>::GuildDoesNotExist)?;
            let role_id =
                Self::role_id(guild_id, request.role_name).ok_or(Error::<T>::RoleDoesNotExist)?;

            match (
                result,
                Members::<T>::contains_key(role_id, &request.account),
            ) {
                (true, false) => {
                    Members::<T>::insert(role_id, &request.account, true);
                    Self::deposit_event(Event::RoleAssigned(
                        request.account,
                        request.guild_name,
                        request.role_name,
                    ));
                }
                (false, true) => {
                    // TODO send locked rewards to requester
                    Members::<T>::remove(role_id, &request.account);
                    Self::deposit_event(Event::RoleStripped(
                        request.account,
                        request.guild_name,
                        request.role_name,
                    ));
                }
                (false, false) => return Err(Error::<T>::AccessDenied.into()),
                (true, true) => {} // nothing happens, requirements are still satisfied
            }

            Ok(())
        }

        #[pallet::call_index(12)]
        #[pallet::weight((0, DispatchClass::Operational))]
        pub fn sudo_remove(
            origin: OriginFor<T>,
            account: T::AccountId,
            guild_name: GuildName,
            role_name: RoleName,
        ) -> DispatchResult {
            ensure_root(origin)?;
            let (role_id, _) = Self::checked_role(&account, &guild_name, &role_name)?;
            Members::<T>::remove(role_id, &account);
            Self::deposit_event(Event::RoleStripped(account, guild_name, role_name));
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn checked_role(
            account: &T::AccountId,
            guild_name: &GuildName,
            role_name: &RoleName,
        ) -> Result<(T::Hash, Role), DispatchError> {
            let guild_id = Self::guild_id(guild_name).ok_or(Error::<T>::GuildDoesNotExist)?;
            let role_id = Self::role_id(guild_id, role_name).ok_or(Error::<T>::RoleDoesNotExist)?;

            // check the requester is registered
            ensure!(
                IdentityPallet::<T>::addresses(account).is_some(),
                Error::<T>::UserNotRegistered
            );

            let role_data = Self::role(role_id).ok_or(Error::<T>::RoleDoesNotExist)?;

            Ok((role_id, role_data))
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

        fn do_create_role(
            signer: T::AccountId,
            guild_name: GuildName,
            role_name: RoleName,
            filter: Option<Filter>,
            requirements: Option<SerializedRequirements>,
        ) -> Result<T::Hash, DispatchError> {
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

        fn evaluate_access(
            signer: T::AccountId,
            filter_access: bool,
            filter_logic: FilterLogic,
            requirements: bool,
            guild_name: GuildName,
            role_name: RoleName,
            role_id: T::Hash,
        ) -> DispatchResult {
            match (filter_access, filter_logic, requirements) {
                // result depends on oracle checks
                (true, FilterLogic::And, true) | (false, FilterLogic::Or, true) => {
                    Self::dispatch_oracle_request(signer, guild_name, role_name)
                }
                // on chain access denied and there's no need for
                // oracle checks
                (false, _, false) | (false, FilterLogic::And, true) => {
                    Err(Error::<T>::AccessDenied.into())
                }
                // on chain access granted and there's no noeed for
                // oracle checks
                (true, _, false) | (true, FilterLogic::Or, true) => {
                    Members::<T>::insert(role_id, &signer, true);
                    Self::deposit_event(Event::RoleAssigned(signer, guild_name, role_name));
                    Ok(())
                }
            }
        }

        fn dispatch_oracle_request(
            signer: T::AccountId,
            guild_name: GuildName,
            role_name: RoleName,
        ) -> DispatchResult {
            let request = AccessCheckRequest {
                requester: signer.clone(),
                account: signer.clone(),
                guild_name,
                role_name,
            };
            let fee = BalanceOf::<T>::unique_saturated_from(
                <T as pallet_oracle::Config>::MinimumFee::get(),
            );
            let pallet_index = <T as frame_system::Config>::PalletInfo::index::<Self>()
                .ok_or(Error::<T>::NoPalletIndex)?;
            <pallet_oracle::Pallet<T>>::initiate_request(
                RawOrigin::Signed(signer).into(),
                pallet_index as u32,
                request.encode(),
                fee,
            )?;
            Ok(())
        }
    }
}
