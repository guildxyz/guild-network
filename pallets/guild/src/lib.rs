#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec as SpVec;

	#[derive(Encode, Decode, Clone, TypeInfo)]
	pub struct Guild<AccountId> {
		owner: AccountId,
		members: SpVec<AccountId>,
	}

	type GuildId = u64;

	#[pallet::storage]
	pub(super) type Guilds<T: Config> =
		StorageMap<_, Blake2_128Concat, GuildId, Guild<T::AccountId>, OptionQuery>;

	#[pallet::config]
	pub trait Config: frame_system::Config {
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
		#[pallet::weight(10_000)]
		pub fn create_guild(origin: OriginFor<T>, guild_id: GuildId) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(!Guilds::<T>::contains_key(&guild_id), Error::<T>::GuildAlreadyExists);
			let guild = Guild { owner: sender.clone(), members: SpVec::new() };
			Guilds::<T>::insert(&guild_id, &guild);
			Self::deposit_event(Event::GuildCreated(sender, guild_id));
			Ok(())
		}

		#[pallet::weight(50_000_000)]
		pub fn join_guild(origin: OriginFor<T>, guild_id: GuildId) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Guilds::<T>::try_mutate(&guild_id, |value| {
				if let Some(guild) = value {
					guild
						.members
						.binary_search(&sender)
						.ok()
						.ok_or(Error::<T>::SignerAlreadyJoined)?;
					guild.members.push(sender.clone());
					Self::deposit_event(Event::GuildJoined(sender, guild_id));
					Ok::<(), DispatchError>(())
				} else {
					Err(Error::<T>::GuildDoesNotExist.into())
				}
			})?;
			Ok(())
		}
	}
}
