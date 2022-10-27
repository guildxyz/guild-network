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
    use frame_support::{
        dispatch::DispatchResult, pallet_prelude::*, sp_runtime::traits::UniqueSaturatedFrom,
        traits::Currency,
    };
    use frame_system::pallet_prelude::*;
    use pallet_chainlink::{CallbackWithParameter, Config as ChainlinkConfig, RequestIdentifier};
    use sp_std::vec::Vec as SpVec;

    type BalanceOf<T> = <<T as pallet_chainlink::Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::Balance;
    type MapId = [u8; 32];

    #[derive(Encode, Decode, Clone, TypeInfo)]
    pub struct Guild<AccountId> {
        pub owner: AccountId,
        pub metadata: SpVec<u8>,
    }

    #[derive(Encode, Decode, Clone, TypeInfo)]
    pub struct JoinRequest<AccountId> {
        pub requester: AccountId,
        pub requester_identities: SpVec<u8>,
        pub guild_id: MapId,
        pub role_id: MapId,
    }

    #[pallet::storage]
    #[pallet::getter(fn next_request_id)]
    pub(super) type NextRequestIdentifier<T: Config> =
        StorageValue<_, RequestIdentifier, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn guild)]
    pub(super) type Guilds<T: Config> =
        StorageMap<_, Blake2_128Concat, MapId, Guild<T::AccountId>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn role)]
    pub(super) type Roles<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        MapId,
        Blake2_128Concat,
        MapId,
        SpVec<u8>, // role metadata
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn member)]
    pub(super) type Members<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, MapId>,
            NMapKey<Blake2_128Concat, MapId>,
            NMapKey<Blake2_128Concat, T::AccountId>,
        ),
        bool,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn join_request)]
    pub(super) type JoinRequests<T: Config> =
        StorageMap<_, Blake2_128Concat, RequestIdentifier, JoinRequest<T::AccountId>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn user_data)]
    pub(super) type UserData<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, SpVec<u8>, OptionQuery>;

    #[pallet::config]
    pub trait Config: ChainlinkConfig<Callback = Call<Self>> + frame_system::Config {
        type WeightInfo: WeightInfo;
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        AccessDenied(T::AccountId, MapId, MapId),
        GuildCreated(T::AccountId, MapId),
        GuildJoined(T::AccountId, MapId, MapId),
        OracleResult(RequestIdentifier, bool),
    }

    #[pallet::error]
    pub enum Error<T> {
        GuildAlreadyExists,
        InvalidResultLength,
        InvalidGuildRole,
        JoinRequestDoesNotExist,
        SignerAlreadyJoined,
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(1000)] //T::WeightInfo::create_guild())]
        pub fn create_guild(
            origin: OriginFor<T>,
            guild_id: MapId,
            metadata: SpVec<u8>,
            roles: SpVec<(MapId, SpVec<u8>)>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(
                !Guilds::<T>::contains_key(guild_id),
                Error::<T>::GuildAlreadyExists
            );
            let guild = Guild {
                owner: sender.clone(),
                metadata,
            };
            Guilds::<T>::insert(guild_id, guild);
            for (role_id, role_metadata) in roles.into_iter() {
                Roles::<T>::insert(guild_id, role_id, role_metadata);
            }
            Self::deposit_event(Event::GuildCreated(sender, guild_id));
            Ok(())
        }

        #[pallet::weight(0)]
        pub fn callback(origin: OriginFor<T>, result: SpVec<u8>) -> DispatchResult {
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
            let access = result[result.len() - 1] != 0; // if last byte is 0 then access = false

            Self::deposit_event(Event::OracleResult(request_id, access));

            let request = if let Some(request) = JoinRequests::<T>::take(request_id) {
                request
            } else {
                return Err(Error::<T>::JoinRequestDoesNotExist.into());
            };

            if !access {
                Self::deposit_event(Event::AccessDenied(
                    request.requester,
                    request.guild_id,
                    request.role_id,
                ));
                // NOTE if we return with an error, all previous computations
                // are reverted it seems, because the join request is not
                // removed
                return Ok(());
            }

            // NOTE request has already been through a filter in `join_request`, i.e.
            // at this point it is safe to assume that the given role id exists within
            // an existing guild
            ensure!(
                !Members::<T>::contains_key((
                    &request.guild_id,
                    &request.role_id,
                    &request.requester
                )),
                Error::<T>::SignerAlreadyJoined
            );

            Members::<T>::insert(
                (&request.guild_id, &request.role_id, &request.requester),
                true,
            );

            if !UserData::<T>::contains_key(&request.requester) {
                UserData::<T>::insert(&request.requester, &request.requester_identities);
            }

            Self::deposit_event(Event::GuildJoined(
                request.requester,
                request.guild_id,
                request.role_id,
            ));

            Ok(())
        }

        #[pallet::weight(1000)] //T::WeightInfo::join_guild())]
        pub fn join_guild(
            origin: OriginFor<T>,
            guild_id: MapId,
            role_id: MapId,
            requester_identities: SpVec<u8>,
            mut request_data: SpVec<u8>,
        ) -> DispatchResult {
            let requester = ensure_signed(origin.clone())?;

            ensure!(
                <Roles<T>>::contains_key(guild_id, role_id),
                Error::<T>::InvalidGuildRole
            );

            let request_id = NextRequestIdentifier::<T>::get();
            // Using `wrapping_add` to start at 0 when it reaches `u64::max_value()`.
            // This means that requests may be overwritten but it requires that at some point
            // at least 2^64 requests are waiting to be served. Since requests also time out
            // after a while this seems extremely unlikely.
            NextRequestIdentifier::<T>::put(request_id.wrapping_add(1));

            let mut request_parameters = requester_identities.clone();
            request_parameters.append(&mut request_data);

            JoinRequests::<T>::insert(
                request_id,
                JoinRequest::<T::AccountId> {
                    requester,
                    requester_identities,
                    guild_id,
                    role_id,
                },
            );

            let call: <T as ChainlinkConfig>::Callback = Call::callback {
                result: SpVec::new(),
            };
            // TODO set unique fee
            let fee = BalanceOf::<T>::unique_saturated_from(100_000_000u32);
            <pallet_chainlink::Pallet<T>>::initiate_request(
                origin,
                0,
                request_parameters,
                fee,
                call,
            )?;

            Ok(())
        }
    }

    impl<T: Config> CallbackWithParameter for Call<T> {
        fn with_result(&self, result: SpVec<u8>) -> Option<Self> {
            match *self {
                Call::callback { result: _ } => Some(Call::callback { result }),
                _ => None,
            }
        }
    }
}
