#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec as SpVec;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ClaimCreated(T::AccountId, SpVec<u8>),
		ClaimRevoked(T::AccountId, SpVec<u8>),
	}

	#[pallet::error]
	pub enum Error<T> {
		ProofAlreadyClaimed,
		NoSuchProof,
		NotProofOwner,
	}

	#[pallet::storage]
	pub(super) type Proofs<T: Config> =
		StorageMap<_, Blake2_128Concat, SpVec<u8>, (T::AccountId, T::BlockNumber), OptionQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1_000)]
		pub fn create_claim(origin: OriginFor<T>, proof: SpVec<u8>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(!Proofs::<T>::contains_key(&proof), Error::<T>::ProofAlreadyClaimed);
			let current_block = <frame_system::Pallet<T>>::block_number();
			Proofs::<T>::insert(&proof, (&sender, current_block));
			Self::deposit_event(Event::ClaimCreated(sender, proof));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn revoke_claim(origin: OriginFor<T>, proof: SpVec<u8>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			if let Some((owner, _)) = Proofs::<T>::get(&proof) {
				ensure!(sender == owner, Error::<T>::NotProofOwner);
				Proofs::<T>::remove(&proof);
				Self::deposit_event(Event::ClaimRevoked(sender, proof));
				Ok(())
			} else {
				Err(Error::<T>::NoSuchProof.into())
			}
		}
	}
}
