#![cfg(test)]

use codec::{Decode, Encode};
use frame_support::{parameter_types, traits::OnFinalize};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

use crate as pallet_chainlink;

use super::*;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Chainlink: pallet_chainlink::{Pallet, Call, Storage, Event<T>},
        Example: example_caller::{Pallet, Call, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

pub(crate) type AccountId = u128;
pub(crate) type BlockNumber = u64;

impl system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
    pub const MinimumFee: u32 = 500;
}

type Balance = u64;

impl pallet_balances::Config for Test {
    type MaxLocks = ();
    type Balance = Balance;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
}

impl pallet_chainlink::Config for Test {
    type Event = Event;
    type Currency = pallet_balances::Pallet<Test>;
    type Callback = example_caller::Call<Test>;
    type ValidityPeriod = ValidityPeriod;
    type MinimumFee = MinimumFee;
}

impl example_caller::Config for Test {
    type Event = Event;
}

parameter_types! {
    pub const ValidityPeriod: u64 = 10;
}

//type Chainlink = pallet_chainlink::Pallet<Test>;

const GENESIS_BALANCE: u64 = 1_000_000_000;

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    pallet_balances::GenesisConfig::<Test> {
        // Total issuance will be 200 with treasury account initialized at ED.
        balances: vec![
            (0, GENESIS_BALANCE),
            (1, GENESIS_BALANCE),
            (2, GENESIS_BALANCE),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    t.into()
}

pub fn last_event() -> tests::Event {
    System::events()
        .into_iter()
        .map(|r| r.event)
        .filter_map(|e| {
            if let Event::Chainlink(inner) = e {
                Some(Event::Chainlink(inner))
            } else {
                None
            }
        })
        .last()
        .unwrap()
}

fn get_minimum_fee() -> u64 {
    <Test as pallet_chainlink::Config>::MinimumFee::get() as u64
}

#[frame_support::pallet]
pub mod example_caller {
    //use super::*;
    use super::pallet_chainlink::CallbackWithParameter;
    use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
    use frame_system::{ensure_root, pallet_prelude::*};

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(1_000_000)]
        pub fn callback(origin: OriginFor<T>, result: Vec<u8>) -> DispatchResult {
            ensure_root(origin)?;
            let full_response: u128 =
                u128::decode(&mut &result[..]).map_err(|_| Error::<T>::DecodingFailed)?;
            let res: u64 = (full_response >> 64) as u64;
            Result::<T>::put(res);
            Ok(())
        }
    }

    #[pallet::storage]
    #[pallet::getter(fn result)]
    pub(super) type Result<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::error]
    pub enum Error<T> {
        DecodingFailed,
    }

    #[pallet::event]
    pub enum Event<T: Config> {}

    impl<T: Config> CallbackWithParameter for Call<T> {
        fn with_result(&self, result: Vec<u8>) -> Option<Self> {
            match *self {
                Call::callback { result: _ } => Some(Call::callback { result }),
                _ => None,
            }
        }
    }
}

#[test]
fn operator_registration_valid() {
    new_test_ext().execute_with(|| {
        // This is required for some reason otherwise the last_event() method fails
        System::set_block_number(1);

        assert!(!<Chainlink>::operator(1));
        assert!(<Chainlink>::register_operator(Origin::signed(1)).is_ok());
        assert_eq!(
            last_event(),
            tests::Event::Chainlink(pallet_chainlink::Event::OperatorRegistered(1))
        );
        assert!(<Chainlink>::operator(1));
    });
}

#[test]
fn operator_registration_invalid_operator_already_registered() {
    new_test_ext().execute_with(|| {
        assert!(<Chainlink>::register_operator(Origin::signed(1)).is_ok());
        assert!(<Chainlink>::operator(1));

        // Operator already registered error
        assert!(<Chainlink>::register_operator(Origin::signed(1)).is_err());
        assert!(<Chainlink>::operator(1));
    });
}

#[test]
fn operator_unregistration_valid() {
    new_test_ext().execute_with(|| {
        // This is required for some reason otherwise the last_event() method fails
        System::set_block_number(1);

        assert!(<Chainlink>::register_operator(Origin::signed(1)).is_ok());
        assert!(<Chainlink>::unregister_operator(Origin::signed(1)).is_ok());
        assert!(!<Chainlink>::operator(1));

        assert_eq!(
            last_event(),
            tests::Event::Chainlink(pallet_chainlink::Event::OperatorUnregistered(1))
        );
    });
}

#[test]
fn operator_unregistration_invalid_unknown_operator() {
    new_test_ext().execute_with(|| {
        // Unknown operator error
        assert!(<Chainlink>::unregister_operator(Origin::signed(1)).is_err());
        assert!(!<Chainlink>::operator(1));
    });
}

#[test]
fn initiate_requests_valid() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        assert!(<Chainlink>::register_operator(Origin::signed(1)).is_ok());
        assert_eq!(
            last_event(),
            tests::Event::Chainlink(pallet_chainlink::Event::OperatorRegistered(1))
        );

        let parameters = ("a", "b");
        let data = parameters.encode();
        assert!(<Chainlink>::initiate_request(
            Origin::signed(2),
            1,
            vec![],
            1,
            data.clone(),
            get_minimum_fee(),
            example_caller::Call::<Test>::callback { result: vec![] }
        )
        .is_ok());
        assert_eq!(
            last_event(),
            tests::Event::Chainlink(pallet_chainlink::Event::OracleRequest(
                1,
                vec![],
                0,
                2,
                1,
                data.clone(),
                "Chainlink.callback".into(),
                get_minimum_fee()
            ))
        );

        let r = <(Vec<u8>, Vec<u8>)>::decode(&mut &data[..]).unwrap().0;
        assert_eq!("a", std::str::from_utf8(&r).unwrap());

        let result: u64 = 10;
        assert!(<Chainlink>::callback(Origin::signed(1), 0, result.encode()).is_ok());

        let expected_answer = Chainlink::prepend_request_id(&mut result.encode(), 0);

        assert_eq!(
            last_event(),
            tests::Event::Chainlink(pallet_chainlink::Event::OracleAnswer(
                1,
                0,
                expected_answer,
                get_minimum_fee()
            ))
        );

        assert_eq!(<example_caller::Result<Test>>::get(), 10);
    });
}

#[test]
fn initiate_requests_invalid_unknown_operator() {
    new_test_ext().execute_with(|| {
        // Unknown operator error
        assert!(<Chainlink>::initiate_request(
            Origin::signed(2),
            1,
            vec![],
            1,
            vec![],
            get_minimum_fee(),
            example_caller::Call::<Test>::callback { result: vec![] }
        )
        .is_err());
    });
}

#[test]
fn initiate_requests_invalid_insufficient_fee() {
    new_test_ext().execute_with(|| {
        assert!(<Chainlink>::register_operator(Origin::signed(1)).is_ok());
        // Insufficient fee error
        assert!(<Chainlink>::initiate_request(
            Origin::signed(2),
            1,
            vec![],
            1,
            vec![],
            get_minimum_fee() - 1,
            example_caller::Call::<Test>::callback { result: vec![] }
        )
        .is_err());
    });
}

#[test]
fn initiate_requests_invalid_insufficient_balance_for_fee() {
    new_test_ext().execute_with(|| {
        assert!(<Chainlink>::register_operator(Origin::signed(1)).is_ok());

        // Insufficient balance error (System error)
        assert!(<Chainlink>::initiate_request(
            Origin::signed(2),
            1,
            vec![],
            1,
            vec![],
            GENESIS_BALANCE + 1,
            example_caller::Call::<Test>::callback { result: vec![] }
        )
        .is_err());
    });
}

#[test]
fn initiate_requests_invalid_wrong_operator() {
    new_test_ext().execute_with(|| {
        assert!(<Chainlink>::register_operator(Origin::signed(1)).is_ok());
        assert!(<Chainlink>::initiate_request(
            Origin::signed(2),
            1,
            vec![],
            1,
            vec![],
            get_minimum_fee(),
            example_caller::Call::<Test>::callback { result: vec![] }
        )
        .is_ok());
        // Wrong operator error
        assert!(<Chainlink>::callback(Origin::signed(3), 0, 10.encode()).is_err());
    });
}

#[test]
fn callback_invalid_unknown_request() {
    new_test_ext().execute_with(|| {
        // Unknown request error
        assert!(<Chainlink>::callback(Origin::signed(1), 0, 10.encode()).is_err());
    });
}

#[test]
fn kill_request() {
    new_test_ext().execute_with(|| {
        assert!(<Chainlink>::register_operator(Origin::signed(1)).is_ok());
        assert!(<Chainlink>::initiate_request(
            Origin::signed(2),
            1,
            vec![],
            1,
            vec![],
            get_minimum_fee(),
            example_caller::Call::<Test>::callback { result: vec![] }
        )
        .is_ok());

        <Chainlink as OnFinalize<u64>>::on_finalize(
            <Test as pallet_chainlink::Config>::ValidityPeriod::get() + 1,
        );
        // Request has been killed, too old
        // Unknown request error
        assert!(<Chainlink>::callback(Origin::signed(1), 0, 10.encode()).is_err());
    });
}
