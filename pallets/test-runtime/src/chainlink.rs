use super::Block;
use super::UncheckedExtrinsic;

frame_support::construct_runtime!(
    pub enum TestRuntime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Chainlink: pallet_chainlink::{Pallet, Call, Storage, Event<T>},
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        TestOperator: pallet_test_operator::{Pallet, Call, Storage, Event<T>},
    }
);

impl pallet_test_operator::Config for TestRuntime {
    type Event = Event;
    type WeightInfo = ();
    type Callback = pallet_test_operator::Call<TestRuntime>;
}
