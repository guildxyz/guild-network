use super::{pallet_oracle, Event, System, TestRuntime};
use frame_support::dispatch::DispatchError;

pub const GENESIS_BALANCE: <TestRuntime as pallet_balances::Config>::Balance = 10;
pub const ACCOUNT_0: <TestRuntime as frame_system::Config>::AccountId = 0;
pub const ACCOUNT_1: <TestRuntime as frame_system::Config>::AccountId = 1;

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut ext = frame_system::GenesisConfig::default()
        .build_storage::<TestRuntime>()
        .unwrap();
    pallet_balances::GenesisConfig::<TestRuntime> {
        balances: vec![(ACCOUNT_0, GENESIS_BALANCE), (ACCOUNT_1, GENESIS_BALANCE)],
    }
    .assimilate_storage(&mut ext)
    .unwrap();

    ext.into()
}

pub fn last_event() -> Event {
    System::events()
        .into_iter()
        .filter_map(|e| {
            if let Event::Oracle(inner) = e.event {
                Some(Event::Oracle(inner))
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
