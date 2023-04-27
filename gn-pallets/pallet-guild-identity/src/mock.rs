pub use crate as pallet_guild_identity;

use frame_support::parameter_types;
use sp_core::H256;
use sp_runtime::testing::Header;
use sp_runtime::traits::{BlakeTwo256, ConstU32, ConstU64, IdentityLookup};

type Balance = u64;
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;
type Block = frame_system::mocking::MockBlock<TestRuntime>;

frame_support::construct_runtime!(
    pub enum TestRuntime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        Balances: pallet_balances,
        GuildIdentity: pallet_guild_identity::{Pallet, Storage, Event<T>},
        Oracle: pallet_oracle::{Pallet, Call, Storage, Event<T>},
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
    }
);

parameter_types! {
    pub const ExistentialDeposit: Balance = 0;
    pub const MinimumFee: Balance = 0;
    pub const MaxLinkedAddresses: u32 = 2;
    pub const MaxLinkedAddressTypes: u32 = 2;
    pub const MaxLinkedIdentities: u32 = 3;
    pub const ValidityPeriod: u64 = 10;
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
    type ReserveIdentifier = [u8; 8];
}

impl pallet_guild_identity::Config for TestRuntime {
    type MaxLinkedAddresses = MaxLinkedAddresses;
    type MaxLinkedAddressTypes = MaxLinkedAddressTypes;
    type MaxLinkedIdentities = MaxLinkedIdentities;
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

impl pallet_oracle::Config for TestRuntime {
    type Currency = pallet_balances::Pallet<TestRuntime>;
    type MaxOperators = ConstU32<10>;
    type MinimumFee = MinimumFee;
    type RuntimeEvent = RuntimeEvent;
    type ValidityPeriod = ValidityPeriod;
    type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut ext: sp_io::TestExternalities = frame_system::GenesisConfig::default()
        .build_storage::<TestRuntime>()
        .unwrap()
        .into();
    ext.execute_with(|| System::set_block_number(1));
    ext
}
