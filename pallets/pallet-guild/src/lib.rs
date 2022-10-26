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
    use sp_std::prelude::*;

    type BalanceOf<T> = <<T as pallet_chainlink::Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::Balance;

    #[derive(Encode, Decode, Clone, TypeInfo)]
    pub struct Guild<AccountId> {
        owner: AccountId,
        members: Vec<AccountId>,
        minimum_balance: u64,
    }

    impl<AccountId> Guild<AccountId> {
        pub fn owner(&self) -> &AccountId {
            &self.owner
        }

        pub fn members(&self) -> &[AccountId] {
            &self.members
        }
    }

    type GuildId = u64;

    #[derive(Encode, Decode, Clone, TypeInfo)]
    pub struct JoinRequest<AccountId> {
        requester: AccountId,
        guild_id: GuildId,
    }

    #[pallet::storage]
    #[pallet::getter(fn request_identifier)]
    pub(super) type NextRequestIdentifier<T: Config> =
        StorageValue<_, RequestIdentifier, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn guilds)]
    pub(super) type Guilds<T: Config> =
        StorageMap<_, Blake2_128Concat, GuildId, Guild<T::AccountId>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn join_requests)]
    pub(super) type JoinRequests<T: Config> =
        StorageMap<_, Blake2_128Concat, RequestIdentifier, JoinRequest<T::AccountId>, OptionQuery>;

    #[pallet::config]
    pub trait Config: ChainlinkConfig<Callback = Call<Self>> + frame_system::Config {
        type WeightInfo: WeightInfo;
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        GuildCreated(T::AccountId, GuildId),
        GuildJoined(T::AccountId, GuildId),
        FailedJoinRequest(T::AccountId, GuildId),
        DecodingComplete(RequestIdentifier, bool),
    }

    #[pallet::error]
    pub enum Error<T> {
        GuildAlreadyExists,
        GuildDoesNotExist,
        JoinRequestDoesNotExist,
        SignerAlreadyJoined,
        InvalidResultLength,
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
            guild_id: GuildId,
            minimum_balance: u64, // TODO this should be u128 (because of gwei)
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(
                !Guilds::<T>::contains_key(guild_id),
                Error::<T>::GuildAlreadyExists
            );
            let guild = Guild {
                owner: sender.clone(),
                members: Vec::new(),
                minimum_balance,
            };
            Guilds::<T>::insert(guild_id, &guild);
            Self::deposit_event(Event::GuildCreated(sender, guild_id));
            Ok(())
        }

        #[pallet::weight(0)]
        pub fn callback(origin: OriginFor<T>, result: Vec<u8>) -> DispatchResult {
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

            Self::deposit_event(Event::DecodingComplete(request_id, access));

            let request = if let Some(request) = JoinRequests::<T>::get(request_id) {
                request
            } else {
                return Err(Error::<T>::JoinRequestDoesNotExist.into());
            };

            if access {
                Guilds::<T>::try_mutate(request.guild_id, |value| {
                    if let Some(guild) = value {
                        if guild.members.binary_search(&request.requester).is_ok() {
                            Err(Error::<T>::SignerAlreadyJoined.into())
                        } else {
                            guild.members.push(request.requester.clone());
                            Self::deposit_event(Event::GuildJoined(
                                request.requester,
                                request.guild_id,
                            ));
                            Ok::<(), DispatchError>(())
                        }
                    } else {
                        Self::deposit_event(Event::FailedJoinRequest(
                            request.requester,
                            request.guild_id,
                        ));
                        Err(Error::<T>::GuildDoesNotExist.into())
                    }
                })?;
            }

            Ok(())
        }

        #[pallet::weight(1000)] //T::WeightInfo::join_guild())]
        pub fn join_guild(
            origin: OriginFor<T>,
            guild_id: GuildId,
            request_parameters: Vec<u8>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin.clone())?;

            ensure!(
                <Guilds<T>>::contains_key(guild_id),
                Error::<T>::GuildDoesNotExist
            );

            let request_id = NextRequestIdentifier::<T>::get();
            // Using `wrapping_add` to start at 0 when it reaches `u64::max_value()`.
            // This means that requests may be overwritten but it requires that at some point
            // at least 2^64 requests are waiting to be served. Since requests also time out
            // after a while this seems extremely unlikely.
            NextRequestIdentifier::<T>::put(request_id.wrapping_add(1));

            JoinRequests::<T>::insert(
                request_id,
                JoinRequest::<T::AccountId> {
                    requester: sender,
                    guild_id,
                },
            );

            let call: <T as ChainlinkConfig>::Callback = Call::callback { result: vec![] };
            // TODO set unique fee
            let fee = BalanceOf::<T>::unique_saturated_from(100_000_000u32);
            <pallet_chainlink::Pallet<T>>::initiate_request(
                origin,
                0,
                request_parameters.encode(),
                fee,
                call,
            )?;

            Ok(())
        }
    }

    impl<T: Config> CallbackWithParameter for Call<T> {
        fn with_result(&self, result: Vec<u8>) -> Option<Self> {
            match *self {
                Call::callback { result: _ } => Some(Call::callback { result }),
                _ => None,
            }
        }
    }
}
