#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;


#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use sp_std::vec::Vec as SpVec;
	//use frame_system::pallet_prelude::*;
	
	#[derive(Encode, Decode, Clone, TypeInfo)]
	pub struct Guild<AccountId> {
		owner: AccountId,
		members: SpVec<AccountId>,
	}

	#[pallet::storage]
	pub(super) type Guilds<T: Config> = 
		StorageMap<_, Blake2_128Concat, u64, Guild<T::AccountId>, OptionQuery>;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type CreateOrigin: EnsureOrigin<Self::Origin>;
		type JoinOrigin: EnsureOrigin<Self::Origin>;
		type MaxMembers: Get<u32>;
	}

	#[pallet::event]
	pub enum Event<T: Config> {
		GuildCreated,
		GuildJoined,
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);
}
