pub use crate as pallet_oracle;

use frame_support::dispatch::{
    DispatchResultWithPostInfo, PostDispatchInfo, UnfilteredDispatchable,
};
use frame_support::pallet_prelude::Pays;
use frame_support::parameter_types;
use parity_scale_codec::{Decode, Encode, EncodeLike};
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::testing::Header;
use sp_runtime::traits::{BlakeTwo256, ConstU32, ConstU64, IdentityLookup};
use sp_std::vec::Vec as SpVec;

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
    type WeightInfo = ();
    type RuntimeEvent = RuntimeEvent;
    type Currency = pallet_balances::Pallet<TestRuntime>;
    type Callback = MockCallback<Self>;
    type ValidityPeriod = ValidityPeriod;
    type MaxOperators = MaxOperators;
    type MinimumFee = MinimumFee;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, TypeInfo, Encode, Decode)]
pub struct MockCallback<T>(pub std::marker::PhantomData<T>);

impl<T> EncodeLike<()> for MockCallback<T> {}

impl<T> pallet_oracle::CallbackWithParameter for MockCallback<T> {
    fn with_result(&self, result: SpVec<u8>) -> Option<Self> {
        if result == [0, 0] {
            None
        } else {
            Some(Self(std::marker::PhantomData))
        }
    }
}

impl UnfilteredDispatchable for MockCallback<TestRuntime> {
    type RuntimeOrigin = <TestRuntime as frame_system::Config>::RuntimeOrigin;
    fn dispatch_bypass_filter(self, _origin: Self::RuntimeOrigin) -> DispatchResultWithPostInfo {
        Ok(PostDispatchInfo {
            actual_weight: None,
            pays_fee: Pays::No,
        })
    }
}

impl MockCallback<TestRuntime> {
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}
