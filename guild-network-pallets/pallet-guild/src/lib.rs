#![cfg_attr(not(feature = "std"), no_std)]
#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
mod test;
pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    use super::weights::WeightInfo;
    use frame_support::traits::Randomness;
    use frame_support::{
        dispatch::DispatchResult, pallet_prelude::*, sp_runtime::traits::UniqueSaturatedFrom,
        traits::Currency,
    };
    use frame_system::pallet_prelude::*;
    use guild_network_common::{GuildName, RoleName};
    use pallet_chainlink::{CallbackWithParameter, Config as ChainlinkConfig, RequestIdentifier};
    use sp_std::vec::Vec as SpVec;

    type BalanceOf<T> = <<T as pallet_chainlink::Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::Balance;

    #[pallet::storage]
    #[pallet::getter(fn nonce)]
    pub type Nonce<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[derive(Encode, Decode, Clone, TypeInfo)]
    pub struct Guild<AccountId> {
        pub owner: AccountId,
        pub metadata: SpVec<u8>,
    }

    #[derive(Encode, Decode, Clone, TypeInfo)]
    pub struct JoinRequest<AccountId> {
        pub requester: AccountId,
        pub requester_identities: SpVec<u8>,
        pub request_data: SpVec<u8>,
        pub guild_name: GuildName,
        pub role_name: RoleName,
    }

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
    pub type Roles<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::Hash,
        SpVec<u8>, // role metadata
        OptionQuery,
    >;

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
    #[pallet::getter(fn join_request)]
    pub type JoinRequests<T: Config> =
        StorageMap<_, Blake2_128Concat, RequestIdentifier, JoinRequest<T::AccountId>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn user_data)]
    pub type UserData<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, SpVec<u8>, OptionQuery>;

    #[pallet::config]
    pub trait Config: ChainlinkConfig<Callback = Call<Self>> + frame_system::Config {
        type WeightInfo: WeightInfo;
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type MyRandomness: Randomness<Self::Hash, Self::BlockNumber>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        AccessDenied(T::AccountId, GuildName, RoleName),
        GuildCreated(T::AccountId, GuildName),
        GuildJoined(T::AccountId, GuildName, RoleName),
        JoinRequestExpired(RequestIdentifier),
        OracleResult(RequestIdentifier, bool),
        SignerAlreadyJoined(T::AccountId, GuildName, RoleName),
    }

    #[pallet::error]
    pub enum Error<T> {
        GuildAlreadyExists,
        InvalidResultLength,
        InvalidGuildRole,
        JoinRequestDoesNotExist,
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    impl<T: Config> Pallet<T> {
        fn get_and_increment_nonce() -> SpVec<u8> {
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

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(1000)] //T::WeightInfo::create_guild())]
        pub fn create_guild(
            origin: OriginFor<T>,
            guild_name: GuildName,
            metadata: SpVec<u8>,
            roles: SpVec<(RoleName, SpVec<u8>)>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(
                !GuildIdMap::<T>::contains_key(guild_name),
                Error::<T>::GuildAlreadyExists
            );

            let guild_id = Self::get_random_uuid();
            GuildIdMap::<T>::insert(guild_name, guild_id);

            let guild = Guild {
                owner: sender.clone(),
                metadata,
            };
            Guilds::<T>::insert(guild_id, guild);

            for (role_name, role_metadata) in roles.into_iter() {
                let role_id = Self::get_random_uuid();
                RoleIdMap::<T>::insert(guild_id, role_name, role_id);
                Roles::<T>::insert(role_id, role_metadata);
            }

            Self::deposit_event(Event::GuildCreated(sender, guild_name));
            Ok(())
        }

        #[pallet::weight(0)]
        pub fn callback(origin: OriginFor<T>, expired: bool, result: SpVec<u8>) -> DispatchResult {
            // NOTE this ensures that only the root can call this function via
            // a callback, see `frame_system::RawOrigin`
            ensure_root(origin)?;

            // NOTE The result is expected to be the request identifier (u64)
            // and a single boolean
            if result.len() != 9 {
                return Err(Error::<T>::InvalidResultLength.into());
            }
            // NOTE unwrap is fine because an u64 can always be decoded from 8
            // bytes and we have already checked the length of the result
            // vector
            let request_id = RequestIdentifier::decode(&mut &result[0..8]).unwrap();
            let access = result[result.len() - 1] != 0;

            if expired {
                JoinRequests::<T>::remove(request_id);
                Self::deposit_event(Event::JoinRequestExpired(request_id));
                return Ok(());
            }

            Self::deposit_event(Event::OracleResult(request_id, access));

            let request = if let Some(request) = JoinRequests::<T>::take(request_id) {
                request
            } else {
                return Err(Error::<T>::JoinRequestDoesNotExist.into());
            };

            if !access {
                Self::deposit_event(Event::AccessDenied(
                    request.requester,
                    request.guild_name,
                    request.role_name,
                ));
                // NOTE if we return with an error, all previous computations
                // are reverted it seems, because the join request is not
                // removed
                return Ok(());
            }

            // NOTE request has already been through a filter in `join_request`, i.e.
            // at this point it is safe to assume that the given role id exists within
            // an existing guild
            let guild_id = Self::guild_id(request.guild_name).unwrap();
            let role_id = Self::role_id(guild_id, request.role_name).unwrap();
            if Members::<T>::contains_key(role_id, &request.requester) {
                Self::deposit_event(Event::SignerAlreadyJoined(
                    request.requester,
                    request.guild_name,
                    request.role_name,
                ));
                return Ok(());
            }

            Members::<T>::insert(role_id, &request.requester, true);

            if !UserData::<T>::contains_key(&request.requester) {
                UserData::<T>::insert(&request.requester, &request.requester_identities);
            }

            Self::deposit_event(Event::GuildJoined(
                request.requester,
                request.guild_name,
                request.role_name,
            ));

            Ok(())
        }

        #[pallet::weight(1000)] //T::WeightInfo::join_guild())]
        pub fn join_guild(
            origin: OriginFor<T>,
            guild_name: GuildName,
            role_name: RoleName,
            requester_identities: SpVec<u8>,
            request_data: SpVec<u8>,
        ) -> DispatchResult {
            let requester = ensure_signed(origin.clone())?;

            ensure!(
                <GuildIdMap<T>>::contains_key(guild_name),
                Error::<T>::InvalidGuildRole
            );
            let guild_id = Self::guild_id(guild_name).unwrap();
            ensure!(
                <RoleIdMap<T>>::contains_key(guild_id, role_name),
                Error::<T>::InvalidGuildRole
            );

            let request_id = NextRequestIdentifier::<T>::get();
            // NOTE Using `wrapping_add` to start at 0 when it reaches
            // `u64::max_value()`. This means that requests may be overwritten
            // but it requires that at some point at least 2^64 requests are
            // waiting to be served. Since requests also time out after a while
            // this seems extremely unlikely.
            NextRequestIdentifier::<T>::put(request_id.wrapping_add(1));

            JoinRequests::<T>::insert(
                request_id,
                JoinRequest::<T::AccountId> {
                    requester,
                    requester_identities,
                    request_data,
                    guild_name,
                    role_name,
                },
            );

            let call: <T as ChainlinkConfig>::Callback = Call::callback {
                expired: false,
                result: SpVec::new(),
            };
            // TODO set unique fee
            let fee = BalanceOf::<T>::unique_saturated_from(100_000_000u32);
            <pallet_chainlink::Pallet<T>>::initiate_request(
                origin,
                call,
                request_id.to_le_bytes().to_vec(),
                fee,
            )?;

            Ok(())
        }
    }

    impl<T: Config> CallbackWithParameter for Call<T> {
        fn with_result(&self, expired: bool, result: SpVec<u8>) -> Option<Self> {
            match self {
                Call::callback { .. } => Some(Call::callback { expired, result }),
                _ => None,
            }
        }
    }
}
