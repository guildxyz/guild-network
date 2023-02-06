use super::{pallet_oracle, RuntimeEvent, System, TestRuntime};
use frame_support::dispatch::DispatchError;

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

    sp_io::TestExternalities::new(storage)
}

pub fn last_event() -> RuntimeEvent {
    System::events()
        .into_iter()
        .filter_map(|e| {
            if let RuntimeEvent::Oracle(inner) = e.event {
                Some(RuntimeEvent::Oracle(inner))
            } else {
                None
            }
        })
        .last()
        .unwrap()
}

pub fn minimum_fee() -> <TestRuntime as pallet_balances::Config>::Balance {
    <TestRuntime as pallet_oracle::Config>::MinimumFee::get()
}

pub fn error_msg<'a>(error: DispatchError) -> &'a str {
    match error {
        DispatchError::Module(module_error) => module_error.message.unwrap(),
        _ => panic!("unexpected error"),
    }
}
