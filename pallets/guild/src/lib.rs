#![cfg_attr(not(feature = "std"), no_std)]
#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]

pub use pallet::*;

// TODO
//#[cfg(feature = "runtime-benchmarks")]
//mod benchmarking;
pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    use super::weights::WeightInfo;
    use frame_support::{
        dispatch::DispatchResult, pallet_prelude::*, sp_runtime::traits::UniqueSaturatedFrom,
        traits::Currency,
    };
    use frame_system::pallet_prelude::*;
    use pallet_chainlink::{CallbackWithParameter, Config as ChainlinkTrait, RequestIdentifier};
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
    pub trait Config: ChainlinkTrait + frame_system::Config {
        type WeightInfo: WeightInfo;
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Callback: From<Call<Self>> + Into<<Self as ChainlinkTrait>::Callback>;
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
            minimum_balance: u64,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(
                !Guilds::<T>::contains_key(&guild_id),
                Error::<T>::GuildAlreadyExists
            );
            let guild = Guild {
                owner: sender.clone(),
                members: Vec::new(),
                minimum_balance,
            };
            Guilds::<T>::insert(&guild_id, &guild);
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
            let request = <JoinRequests<T>>::get(&request_id).unwrap();

            ensure!(
                <Guilds<T>>::contains_key(request.guild_id),
                Error::<T>::GuildDoesNotExist
            );
            // Unwrap is fine here because we check its existence previously
            let guild = <Guilds<T>>::get(&request.guild_id).unwrap();

            if eth_balance >= guild.minimum_balance {
                Guilds::<T>::try_mutate(&request.guild_id, |value| {
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
            eth_address: Vec<u8>,
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
            let call: <T as Config>::Callback = Call::callback { result: vec![] }.into();
            let spec_id = vec![0];

            let fee = BalanceOf::<T>::unique_saturated_from(100_000_000u32);

            <pallet_chainlink::Pallet<T>>::initiate_request(
                origin,
                operator,
                spec_id,
                0,
                parameters.encode(),
                fee,
                call.into(),
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

/*
#[cfg(test)]
mod test {
    use super::*;
    use crate as pallet_guild;

    use sp_core::H256;
    use sp_runtime::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
    };

    use frame_support::{
        assert_noop, assert_ok,
        traits::{ConstU32, ConstU64},
    };

    type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
    type Block = frame_system::mocking::MockBlock<Test>;

    frame_support::construct_runtime!(
        pub enum Test where
            Block = Block,
            NodeBlock = Block,
            UncheckedExtrinsic = UncheckedExtrinsic,
        {
            System: frame_system,
            Guild: pallet_guild,
        }
    );

    impl frame_system::Config for Test {
        type BaseCallFilter = frame_support::traits::Everything;
        type BlockWeights = ();
        type BlockLength = ();
        type DbWeight = ();
        type Origin = Origin;
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Call = Call;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = Event;
        type BlockHashCount = ConstU64<250>;
        type Version = ();
        type PalletInfo = PalletInfo;
        type AccountData = ();
        type OnNewAccount = ();
        type OnKilledAccount = ();
        type SystemWeightInfo = ();
        type SS58Prefix = ();
        type OnSetCode = ();
        type MaxConsumers = ConstU32<16>;
    }

    impl Config for Test {
        type Event = Event;
        type WeightInfo = ();
    }


    #[test]
    fn guild_interactions_work() {
        let mut ext: sp_io::TestExternalities =
            frame_system::GenesisConfig::default().build_storage::<Test>().unwrap().into();
        ext.execute_with(|| {
            assert_ok!(Guild::create_guild(Origin::signed(4), 444));
            assert!(Guild::guilds(444).is_some());
            assert_ok!(Guild::join_guild(Origin::signed(4), 444));
            assert_eq!(Guild::guilds(444).unwrap().members().len(), 1);
            assert_eq!(Guild::guilds(444).unwrap().members()[0], 4);
            assert_noop!(
                Guild::create_guild(Origin::signed(4), 444),
                Error::<Test>::GuildAlreadyExists
            );
            assert_noop!(
                Guild::create_guild(Origin::signed(5), 444),
                Error::<Test>::GuildAlreadyExists
            );
            assert_noop!(
                Guild::join_guild(Origin::signed(4), 444),
                Error::<Test>::SignerAlreadyJoined
            );
            assert_ok!(Guild::join_guild(Origin::signed(5), 444));
            assert_ok!(Guild::join_guild(Origin::signed(6), 444));
            assert_ok!(Guild::join_guild(Origin::signed(7), 444));
            assert_ok!(Guild::join_guild(Origin::signed(8), 444));
            assert_eq!(Guild::guilds(444).unwrap().members().len(), 5);
            assert_noop!(
                Guild::join_guild(Origin::signed(7), 444),
                Error::<Test>::SignerAlreadyJoined
            );
            assert_noop!(
                Guild::join_guild(Origin::signed(8), 446),
                Error::<Test>::GuildDoesNotExist
            );
            assert_ok!(Guild::create_guild(Origin::signed(1), 446));
            assert_ok!(Guild::join_guild(Origin::signed(8), 446));
        });
    }
}
*/
