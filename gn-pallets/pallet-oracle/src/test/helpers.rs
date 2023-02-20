use super::{pallet_oracle, RuntimeEvent, System, TestRuntime};
use frame_support::dispatch::DispatchError;

pub fn last_event() -> pallet_oracle::Event<TestRuntime> {
    System::events()
        .into_iter()
        .filter_map(|e| {
            if let RuntimeEvent::Oracle(inner) = e.event {
                Some(inner)
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
        DispatchError::BadOrigin => "BadOrigin",
        _ => panic!("unexpected error"),
    }
}
