// This is to suppress weird unused warnings when run with the
// `runtime-benchmarks` feature flag enabled. It probably emanates from the
// `impl_benchmark_test_suite` macro.
#![cfg_attr(feature = "runtime-benchmarks", allow(unused))]
pub use crate as pallet_oracle;

use frame_support::parameter_types;
use sp_core::H256;
use sp_runtime::testing::Header;
use sp_runtime::traits::{BlakeTwo256, ConstU32, ConstU64, IdentityLookup};

type Balance = u64;
type Block = frame_system::mocking::MockBlock<TestRuntime>;
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;

frame_support::construct_runtime!(
    pub enum TestRuntime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        Balances: pallet_balances::{Pallet, Event<T>},
        Oracle: pallet_oracle::{Pallet, Storage, Event<T>},
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
    }
);

parameter_types! {
    pub const ExistentialDeposit: Balance = 1;
    pub const MinimumFee: Balance = 1;
    pub const ValidityPeriod: u64 = 10;
    pub const MaxOperators: u32 = 4;
}

impl frame_system::Config for TestRuntime {
    type BaseCallFilter = frame_support::traits::Everything;
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type DbWeight = ();
    type BlockWeights = ();
    type BlockLength = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

impl pallet_balances::Config for TestRuntime {
    type MaxLocks = ();
    type Balance = Balance;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxReserves = ();
    type ReserveIdentifier = ();
}

impl pallet_oracle::Config for TestRuntime {
    type Currency = pallet_balances::Pallet<TestRuntime>;
    type MaxOperators = MaxOperators;
    type MinimumFee = MinimumFee;
    type RuntimeEvent = RuntimeEvent;
    type ValidityPeriod = ValidityPeriod;
    type WeightInfo = ();
}

pub const GENESIS_BALANCE: <TestRuntime as pallet_balances::Config>::Balance = 10;
pub const ACCOUNT_0: <TestRuntime as frame_system::Config>::AccountId = 0;
pub const ACCOUNT_1: <TestRuntime as frame_system::Config>::AccountId = 1;

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::default()
        .build_storage::<TestRuntime>()
        .unwrap();
    pallet_balances::GenesisConfig::<TestRuntime> {
        balances: vec![(ACCOUNT_0, GENESIS_BALANCE), (ACCOUNT_1, GENESIS_BALANCE)],
    }
    .assimilate_storage(&mut storage)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(storage);
    ext.execute_with(|| System::set_block_number(1));
    ext
}
