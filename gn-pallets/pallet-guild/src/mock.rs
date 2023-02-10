pub use crate as pallet_guild;

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
        Guild: pallet_guild::{Pallet, Storage, Event<T>},
        Oracle: pallet_oracle::{Pallet, Call, Storage, Event<T>},
        RandomnessCollectiveFlip: pallet_randomness_collective_flip,
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
    }
);

parameter_types! {
    pub const ExistentialDeposit: Balance = 0;
    pub const MinimumFee: Balance = 0;
    pub const MaxIdentities: u8 = 3;
    pub const MaxRolesPerGuild: u32 = 10;
    pub const MaxReqsPerRole: u32 = 10;
    pub const MaxSerializedReqLen: u32 = 10;
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

impl pallet_guild::Config for TestRuntime {
    type MaxIdentities = MaxIdentities;
    type MaxRolesPerGuild = MaxRolesPerGuild;
    type MaxReqsPerRole = MaxReqsPerRole;
    type MaxSerializedReqLen = MaxSerializedReqLen;
    type MyRandomness = RandomnessCollectiveFlip;
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

impl pallet_oracle::Config for TestRuntime {
    type Currency = pallet_balances::Pallet<TestRuntime>;
    type Callback = pallet_guild::Call<TestRuntime>;
    type MinimumFee = MinimumFee;
    type RuntimeEvent = RuntimeEvent;
    type ValidityPeriod = ValidityPeriod;
    type WeightInfo = ();
}

impl pallet_randomness_collective_flip::Config for TestRuntime {}
