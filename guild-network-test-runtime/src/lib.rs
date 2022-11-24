#[macro_export]
macro_rules! test_runtime {
    ($name:ident, $pallet:ident) => {
        use frame_support::parameter_types;
        use sp_core::H256;
        use sp_runtime::{
            testing::Header,
            traits::{BlakeTwo256, IdentityLookup},
        };

        frame_support::construct_runtime!(
            pub enum TestRuntime where
                Block = Block,
                NodeBlock = Block,
                UncheckedExtrinsic = UncheckedExtrinsic,
            {
                Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
                Chainlink: pallet_chainlink::{Pallet, Call, Storage, Event<T>},
                $name: $pallet::{Pallet, Call, Storage, Event<T>},
                System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
                RandomnessCollectiveFlip: pallet_randomness_collective_flip,
            }

        );

        impl pallet_randomness_collective_flip::Config for TestRuntime {}

        pub type AccountId = u64;
        pub type Balance = u64;
        pub type Block = frame_system::mocking::MockBlock<TestRuntime>;
        pub type BlockNumber = u64;
        pub type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;

        parameter_types! {
            pub const BlockHashCount: u64 = 250;
            pub const ExistentialDeposit: u64 = 1;
            pub const MinimumFee: u32 = 500;
            pub const SS58Prefix: u8 = 42;
            pub const ValidityPeriod: u64 = 10;
        }

        impl frame_system::Config for TestRuntime {
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

        impl pallet_balances::Config for TestRuntime {
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

        impl pallet_chainlink::Config for TestRuntime {
            type Event = Event;
            type Currency = pallet_balances::Pallet<TestRuntime>;
            type Callback = $pallet::Call<TestRuntime>;
            type ValidityPeriod = ValidityPeriod;
            type MinimumFee = MinimumFee;
            type WeightInfo = ();
        }

        impl $pallet::Config for TestRuntime {
            type WeightInfo = ();
            type Event = Event;
            type MyRandomness = RandomnessCollectiveFlip;
        }

        pub const GENESIS_BALANCE: u64 = 1_000_000_000;

        pub fn new_test_runtime() -> sp_io::TestExternalities {
            let mut t = frame_system::GenesisConfig::default()
                .build_storage::<TestRuntime>()
                .unwrap();
            pallet_balances::GenesisConfig::<TestRuntime> {
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
    }
}
