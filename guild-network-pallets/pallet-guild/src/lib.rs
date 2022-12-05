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
    use guild_network_common::*;
    use pallet_chainlink::{CallbackWithParameter, Config as ChainlinkConfig};
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
        GuildCreated(T::AccountId, GuildName),
        GuildJoined(T::AccountId, GuildName, RoleName),
    }

    #[pallet::error]
    pub enum Error<T> {
        AccessDenied,
        GuildAlreadyExists,
        GuildDoesNotExist,
        RoleDoesNotExist,
        InvalidOracleAnswer,
        InvalidOracleRequest,
        JoinRequestDoesNotExist,
        UserAlreadyJoined,
        UserAlreadyRegistered,
        UserNotRegistered,
        CodecError,
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
        #[pallet::weight(1000)] //T::WeightInfo::register())]
        pub fn register(origin: OriginFor<T>, data: RequestData) -> DispatchResult {
            let requester = ensure_signed(origin.clone())?;

            ensure!(
                !<UserData<T>>::contains_key(requester),
                Error::<T>::UserAlreadyRegistered
            );

            // check data variant
            ensure!(
                matches_variant(&data, &RequestData::Register(SpVec::new())),
                Error::<T>::InvalidOracleRequest
            );

            let request = Request::<T::AccountId> { requester, data };

            let call: <T as ChainlinkConfig>::Callback = Call::callback {
                result: SpVec::new(),
            };
            // TODO set unique fee
            let fee = BalanceOf::<T>::unique_saturated_from(100_000_000u32);
            <pallet_chainlink::Pallet<T>>::initiate_request(origin, call, request.encode(), fee)?;

            Ok(())
        }

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

        #[pallet::weight(1000)] //T::WeightInfo::join_guild())]
        pub fn join_guild(origin: OriginFor<T>, data: RequestData) -> DispatchResult {
            let requester = ensure_signed(origin.clone())?;

            ensure!(
                <UserData<T>>::contains_key(requester),
                Error::<T>::UserNotRegistered
            );

            ensure!(
                <GuildIdMap<T>>::contains_key(guild_name),
                Error::<T>::GuildDoesNotExist
            );
            // NOTE unwrap is fine because of the ensure check above
            let guild_id = Self::guild_id(guild_name).unwrap();

            ensure!(
                <RoleIdMap<T>>::contains_key(guild_id, role_name),
                Error::<T>::RoleDoesNotExist
            );
            // NOTE unwrap is fine because of the ensure check above
            let role_id = Self::role_id(guild_id, role_name).unwrap();

            ensure!(
                !Members::<T>::contains_key(role_id, &join_request.requester),
                Error::<T>::UserAlreadyJoined
            );

            // check data variant
            match data {
                RequestData::Join { .. } => {}
                _ => return Err(Error::<T>::InvalidOracleRequest.into()),
            }

            let request = Request::<T::AccountId> { requester, data };

            let call: <T as ChainlinkConfig>::Callback = Call::callback {
                result: SpVec::new(),
            };
            // TODO set unique fee
            let fee = BalanceOf::<T>::unique_saturated_from(100_000_000u32);
            <pallet_chainlink::Pallet<T>>::initiate_request(origin, call, request.encode(), fee)?;

            Ok(())
        }

        #[pallet::weight(0)]
        pub fn callback(origin: OriginFor<T>, result: SpVec<u8>) -> DispatchResult {
            // NOTE this ensures that only the root can call this function via
            // a callback, see `frame_system::RawOrigin`
            ensure_root(origin)?;

            // cannot wrap codec::Error in this error type because
            // it doesn't implement the required traits
            let answer = pallet_chainlink::OracleAnswer::decode(&mut result.as_slice())
                .map_err(|_| Error::<T>::CodecError)?;

            ensure!(answer.result.len() == 1, Error::<T>::InvalidOracleAnswer);

            let access = answer.result[0] == 1;
            // if we deposit and event here, it does not appear if an error is
            // returned
            ensure!(access, Error::<T>::AccessDenied);

            let join_request = JoinRequest::<T::AccountId>::decode(&mut answer.data.as_slice())
                .map_err(|_| Error::<T>::CodecError)?;

            let guild_id =
                Self::guild_id(join_request.guild_name).ok_or(Error::<T>::GuildDoesNotExist)?;
            let role_id = Self::role_id(guild_id, join_request.role_name)
                .ok_or(Error::<T>::RoleDoesNotExist)?;

            ensure!(
                !Members::<T>::contains_key(role_id, &join_request.requester),
                Error::<T>::UserAlreadyJoined
            );

            Members::<T>::insert(role_id, &join_request.requester, true);

            if !UserData::<T>::contains_key(&join_request.requester) {
                UserData::<T>::insert(&join_request.requester, &join_request.requester_identities);
            }

            Self::deposit_event(Event::GuildJoined(
                join_request.requester,
                join_request.guild_name,
                join_request.role_name,
            ));

            Ok(())
        }
    }

    impl<T: Config> CallbackWithParameter for Call<T> {
        fn with_result(&self, result: SpVec<u8>) -> Option<Self> {
            match self {
                Call::callback { .. } => Some(Call::callback { result }),
                _ => None,
            }
        }
    }
}
