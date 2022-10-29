//! # A pallet to interact with Chainlink nodes
//!
//! \## Overview
//!
//! `pallet-chainlink` allows to request external data from chainlink operators. This is done by
//! emitting a well-known event (`OracleEvent`) embedding all required data. This event is listened
//! by operators that in turns call back the `callback` function with the associated data.
//!
//! To initiate a request, users call `initiate_request` with the relevant details, the `operator`
//! AccountId and the `fee` they agree to spend to get the result.
//!
//! To be valid, an operator must register its AccountId first hand via `register_operator`.
//!
//! \## Terminology
//! Operator: a member of chainlink that provides result to requests, in exchange of a fee payment
//! Request: details about what the user expects as result. Must match a Specification supported by
//! an identified Operator Fee: the amount of token a users pays to an operator

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod weights;

pub use pallet::*;

use codec::Codec;
use frame_support::{
    dispatch::DispatchResult,
    traits::{BalanceStatus, Currency, Get, ReservableCurrency, UnfilteredDispatchable},
    Parameter,
};
use sp_std::{prelude::*, vec::Vec as SpVec};
use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{ensure, pallet_prelude::*};
    use frame_system::{ensure_signed, pallet_prelude::*};

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type WeightInfo: WeightInfo;
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Currency: ReservableCurrency<Self::AccountId>;
        // A reference to an Extrinsic that can have a result injected. Used as Chainlink callback
        type Callback: Parameter
            + UnfilteredDispatchable<Origin = Self::Origin>
            + Codec
            + Eq
            + CallbackWithParameter;

        // NOTE: The following two types could be `const`
        // Period during which a request is valid
        type ValidityPeriod: Get<Self::BlockNumber>;
        // Minimum fee paid for all requests to disincentivize spam requests
        type MinimumFee: Get<u32>;
    }

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    // Uniquely identify an operator among the registered Operators
    pub type OperatorIdentifier = u64;
    // Uniquely identify a request for a considered Operator
    pub type RequestIdentifier = u64;
    // The version of the serialized data format
    pub type DataVersion = u64;

    // A trait allowing to inject Operator results back into the specified Call
    pub trait CallbackWithParameter {
        fn with_result(&self, result: SpVec<u8>) -> Option<Self>
        where
            Self: core::marker::Sized;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// No oracle operator has registered yet
        NoRegisteredOperators,
        /// Manipulating an unknown operator
        UnknownOperator,
        /// Manipulating an unknown request
        UnknownRequest,
        /// Not the expected operator
        WrongOperator,
        /// An operator is already registered.
        OperatorAlreadyRegistered,
        /// Callback cannot be deserialized
        UnknownCallback,
        /// Fee provided does not match minimum required fee
        InsufficientFee,
        /// Request has already been served by the operator
        RequestAlreadyServed,
        /// Reserved balance is less than the specified fee for the request
        InsufficientReservedBalance,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A request has been accepted. Corresponding fee payment is reserved
        OracleRequest(
            T::AccountId,
            RequestIdentifier,
            T::AccountId,
            DataVersion,
            SpVec<u8>,
            SpVec<u8>,
            BalanceOf<T>,
        ),
        /// A request has been answered. Corresponding fee payment is transferred
        OracleAnswer(T::AccountId, RequestIdentifier, SpVec<u8>, BalanceOf<T>),
        /// A new operator has been registered
        OperatorRegistered(T::AccountId),
        /// An existing operator has been unregistered
        OperatorDeregistered(T::AccountId),
        /// A request didn't receive any result in time
        KillRequest(RequestIdentifier),
        KillRequestFailed(RequestIdentifier),
    }

    /// Stores registered operator addresses in a Vector.
    ///
    /// These could be stored in either a Vector or a Map and the reason why a
    /// Vector was implemented is the following: it is easier to delegate
    /// operator addresses randomly from a Vector than from a Map. The
    /// trade-off is that the storage vector has to be iterated over whenever
    /// an operator registers/deregisters. However, these events are
    /// anticipated to be much less frequent than user request events.
    #[pallet::storage]
    #[pallet::getter(fn operators)]
    pub type Operators<T: Config> = StorageValue<_, SpVec<T::AccountId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn request_identifier)]
    pub type NextRequestIdentifier<T: Config> = StorageValue<_, RequestIdentifier, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn next_operator)]
    pub type NextOperator<T: Config> = StorageValue<_, OperatorIdentifier, ValueQuery>;

    #[derive(Encode, Decode, Clone, TypeInfo)]
    pub struct GenericRequest<AccountId, Callback, BlockNumber, BalanceOf> {
        requester: AccountId,
        operator: AccountId,
        callback: Callback,
        block_number: BlockNumber,
        fee: BalanceOf,
    }

    pub type Request<T> = GenericRequest<
        <T as frame_system::Config>::AccountId,
        <T as Config>::Callback,
        <T as frame_system::Config>::BlockNumber,
        BalanceOf<T>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn request)]
    pub type Requests<T: Config> =
        StorageMap<_, Blake2_128Concat, RequestIdentifier, Request<T>, OptionQuery>;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register a new Operator.
        ///
        /// Fails with `OperatorAlreadyRegistered` if this Operator (identified
        /// by `origin`) has already been registered.
        #[pallet::weight(T::WeightInfo::register_operator())]
        pub fn register_operator(origin: OriginFor<T>) -> DispatchResult {
            let who: <T as frame_system::Config>::AccountId = ensure_signed(origin)?;

            Operators::<T>::try_mutate(|operators| {
                if operators.binary_search(&who).is_ok() {
                    Err(Error::<T>::OperatorAlreadyRegistered.into())
                } else {
                    operators.push(who.clone());
                    Self::deposit_event(Event::OperatorRegistered(who));
                    Ok(())
                }
            })
        }

        /// Deregisters an already registered Operator
        #[pallet::weight(T::WeightInfo::deregister_operator())]
        pub fn deregister_operator(origin: OriginFor<T>) -> DispatchResult {
            let who: <T as frame_system::Config>::AccountId = ensure_signed(origin)?;

            Operators::<T>::try_mutate(|operators| {
                if let Ok(index) = operators.binary_search(&who) {
                    Self::deposit_event(Event::OperatorDeregistered(operators.remove(index)));
                    Ok(())
                } else {
                    Err(Error::<T>::UnknownOperator.into())
                }
            })
        }

        /// Hint specified Operator (via its `AccountId`) of a request to be
        /// performed.
        ///
        /// Request details are encapsulated in `data` which must be
        /// SCALE encoded. If provided fee is sufficient, Operator must send
        /// back the request result in `callback` Extrinsic which then will
        /// dispatch back to the request originator callback identified by
        /// `callback`. The fee is `reserved` and only actually transferred
        /// when the result is provided in the callback. Operators are expected
        /// to listen to `OracleRequest` events. This event contains all the
        /// required information to perform the request and provide back
        /// the result.
        // TODO check weight
        #[pallet::weight(50_000)]
        pub fn initiate_request(
            origin: OriginFor<T>,
            data_version: DataVersion,
            data: Vec<u8>,
            fee: BalanceOf<T>,
            callback: <T as Config>::Callback,
        ) -> DispatchResult {
            let who: <T as frame_system::Config>::AccountId = ensure_signed(origin)?;

            let operators = Operators::<T>::get();
            if operators.is_empty() {
                return Err(Error::<T>::NoRegisteredOperators.into());
            }
            let next_operator = NextOperator::<T>::get();
            let operator = operators[next_operator as usize % operators.len()].clone();

            NextOperator::<T>::put(next_operator.wrapping_add(1));

            // Currency::minimum_balance() is equivalent to ExistentialDeposit in the
            // pallet_balances config of the runtime
            // ensure!(fee > T::Currency::minimum_balance(), Error::<T>::InsufficientFee);

            // NOTE: this might not be necessary since it seems that reserved
            // tokens are only moved from the `free` balance of an account and
            // it is not stored in a totally new account However, a minimum
            // amount of fee is a good idea to disincentivize spam requests
            ensure!(
                fee >= T::MinimumFee::get().into(),
                Error::<T>::InsufficientFee
            );

            T::Currency::reserve(&who, fee)?;

            let request_id = NextRequestIdentifier::<T>::get();
            // Using `wrapping_add` to start at 0 when it reaches `u64::max_value()`.
            // This means that requests may be overwritten but it requires that at some point
            // at least 2^64 requests are waiting to be served. Since requests also time out
            // after a while this seems extremely unlikely.
            NextRequestIdentifier::<T>::put(request_id.wrapping_add(1));

            // NOTE: This does not validate the request for any block number.
            // It only serves as a timestamp for the ValidityPeriod check.
            let now = frame_system::Pallet::<T>::block_number();

            Requests::<T>::insert(
                request_id,
                Request::<T> {
                    requester: who.clone(),
                    operator: operator.clone(),
                    callback,
                    block_number: now,
                    fee,
                },
            );

            Self::deposit_event(Event::OracleRequest(
                operator,
                request_id,
                who,
                data_version,
                data,
                "Chainlink.callback".into(),
                fee,
            ));

            Ok(())
        }

        /// The callback used to be notified of all Operators results.
        ///
        /// Only the Operator responsible for an identified request can notify
        /// back the result. Result is then dispatched back to the originator's
        /// callback. The fee reserved during `initiate_request` is transferred
        /// as soon as this callback is called.
        //TODO check weight
        #[pallet::weight(50_000)]
        pub fn callback(
            origin: OriginFor<T>,
            request_id: RequestIdentifier,
            mut result: Vec<u8>,
        ) -> DispatchResult {
            let who: <T as frame_system::Config>::AccountId = ensure_signed(origin)?;

            ensure!(
                <Requests<T>>::contains_key(request_id),
                Error::<T>::UnknownRequest
            );
            // Unwrap is fine here because we check its existence in the previous line
            let request = <Requests<T>>::get(request_id).unwrap();
            ensure!(request.operator == who, Error::<T>::WrongOperator);

            // NOTE: This should not be possible technically but it is here to be safe
            ensure!(
                request.fee <= T::Currency::reserved_balance(&request.requester),
                Error::<T>::InsufficientReservedBalance
            );

            // NOTE: While `repatriate_reserved` only moves UP TO the amount
            // passed, the currency cannot be moved by a different pallet and
            // we made sure to reserve the exact same amount of balance in the
            // initiate_request call so I believe this is fine.
            // NOTE: BalanceStatus::Free means that it is transferred to the
            // Free balance of the operator
            T::Currency::repatriate_reserved(
                &request.requester,
                &request.operator,
                request.fee,
                BalanceStatus::Free,
            )?;

            let mut prepended_response = request_id.encode();
            prepended_response.append(&mut result);

            // Dispatch the result to the original callback registered by the caller
            let callback = request
                .callback
                .with_result(prepended_response.clone())
                .ok_or(Error::<T>::UnknownCallback)?;
            callback
                .dispatch_bypass_filter(frame_system::RawOrigin::Root.into())
                .map_err(|e| e.error)?;

            // Remove the request from the queue
            Requests::<T>::remove(request_id);

            Self::deposit_event(Event::OracleAnswer(
                request.operator,
                request_id,
                prepended_response,
                request.fee,
            ));

            Ok(())
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        // Identify requests that are considered dead and remove them
        fn on_finalize(n: T::BlockNumber) {
            // NOTE according to the docs of storage maps if a map is modified
            // while iterating over it, we get undefined behaviour, thus we need
            // to iterate over it first, collect expired request_ids and iterate
            // over them while removing the respective requests from the map.
            let request_ids = Requests::<T>::iter()
                .map(|(id, _)| id)
                .collect::<Vec<RequestIdentifier>>();
            for request_id in &request_ids {
                // NOTE unwrap is fine here because we collected existing keys
                let request = Requests::<T>::take(request_id).unwrap();
                if n > request.block_number + T::ValidityPeriod::get() {
                    let mut prepended_response = request_id.encode();
                    prepended_response.push(u8::MAX);

                    if let Some(callback) = request.callback.with_result(prepended_response.clone())
                    {
                        if callback
                            .dispatch_bypass_filter(frame_system::RawOrigin::Root.into())
                            .is_ok()
                        {
                            Self::deposit_event(Event::KillRequest(*request_id));
                        } else {
                            Self::deposit_event(Event::KillRequestFailed(*request_id));
                        }
                    }
                }
            }
        }
    }
}
