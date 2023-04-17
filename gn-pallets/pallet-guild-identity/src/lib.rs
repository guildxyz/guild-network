#![cfg_attr(not(feature = "std"), no_std)]
#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(unused_crate_dependencies)]

#[frame_support::pallet]
pub mod pallet {
    #[pallet::storage]
    #[pallet::getter(fn identity)]
    pub type Identity<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        u8,
        Identity,
        OptionQuery,
    >;
}
