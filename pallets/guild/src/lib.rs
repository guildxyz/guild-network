#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;

#[frame_support::pallet]
pub mod pallet {
	use super::weights::WeightInfo;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec as SpVec;

	#[derive(Encode, Decode, Clone, TypeInfo)]
	pub struct Guild<AccountId> {
		owner: AccountId,
		members: SpVec<AccountId>,
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

	#[pallet::storage]
	#[pallet::getter(fn guilds)]
	pub(super) type Guilds<T: Config> =
		StorageMap<_, Blake2_128Concat, GuildId, Guild<T::AccountId>, OptionQuery>;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type WeightInfo: WeightInfo;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		GuildCreated(T::AccountId, GuildId),
		GuildJoined(T::AccountId, GuildId),
	}

	#[pallet::error]
	pub enum Error<T> {
		GuildAlreadyExists,
		GuildDoesNotExist,
		SignerAlreadyJoined,
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::create_guild())]
		pub fn create_guild(origin: OriginFor<T>, guild_id: GuildId) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(!Guilds::<T>::contains_key(&guild_id), Error::<T>::GuildAlreadyExists);
			let guild = Guild { owner: sender.clone(), members: SpVec::new() };
			Guilds::<T>::insert(&guild_id, &guild);
			Self::deposit_event(Event::GuildCreated(sender, guild_id));
			Ok(())
		}

		#[pallet::weight(T::WeightInfo::join_guild())]
		pub fn join_guild(origin: OriginFor<T>, guild_id: GuildId) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Guilds::<T>::try_mutate(&guild_id, |value| {
				if let Some(guild) = value {
					if guild.members.binary_search(&sender).is_ok() {
						Err(Error::<T>::SignerAlreadyJoined.into())
					} else {
						guild.members.push(sender.clone());
						Self::deposit_event(Event::GuildJoined(sender, guild_id));
						Ok::<(), DispatchError>(())
					}
				} else {
					Err(Error::<T>::GuildDoesNotExist.into())
				}
			})?;
			Ok(())
		}
	}
}

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
