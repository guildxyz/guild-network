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
        DecodingComplete(u128, RequestIdentifier, u64),
    }

    #[pallet::error]
    pub enum Error<T> {
        GuildAlreadyExists,
        GuildDoesNotExist,
        JoinRequestDoesNotExist,
        SignerAlreadyJoined,
        DecodingFailed,
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(T::WeightInfo::create_guild())]
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
            ensure_root(origin)?;

            // The result is expected to be two SCALE encoded `u64`s
            // NOTE: if this does not work, decode a single u128 and segment it
            let everything: u128 =
                u128::decode(&mut &result[..]).map_err(|_| Error::<T>::DecodingFailed)?;
            let request_id: RequestIdentifier = everything as u64;
            let eth_balance: u64 = (everything >> 64) as u64;

            Self::deposit_event(Event::DecodingComplete(everything, request_id, eth_balance));

            ensure!(
                <JoinRequests<T>>::contains_key(request_id),
                Error::<T>::JoinRequestDoesNotExist
            );
            // Unwrap is fine here because we check its existence previously
            let request = <JoinRequests<T>>::get(request_id).unwrap();

            ensure!(
                <Guilds<T>>::contains_key(request.guild_id),
                Error::<T>::GuildDoesNotExist
            );
            // Unwrap is fine here because we check its existence previously
            let guild = <Guilds<T>>::get(request.guild_id).unwrap();

            if eth_balance >= guild.minimum_balance {
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
                        Err(Error::<T>::GuildDoesNotExist.into())
                    }
                })?;
            }

            Ok(())
        }

        #[pallet::weight(T::WeightInfo::join_guild())]
        pub fn join_guild(
            origin: OriginFor<T>,
            guild_id: GuildId,
            eth_address: Vec<u8>, // TODO could this be a fixed length array?
            operator: T::AccountId,
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

            let parameters = eth_address;
            let call: <T as ChainlinkConfig>::Callback = Call::callback { result: vec![] };
            let spec_id = vec![0];

            // TODO set unique fee
            let fee = BalanceOf::<T>::unique_saturated_from(100_000_000u32);

            <pallet_chainlink::Pallet<T>>::initiate_request(
                origin,
                operator,
                spec_id,
                0,
                parameters.encode(),
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
