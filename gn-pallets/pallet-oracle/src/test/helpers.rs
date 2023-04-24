use super::{pallet_oracle, RuntimeEvent, System, TestRuntime};
pub use frame_support::dispatch::DispatchError;

pub type OracleEvent = pallet_oracle::Event<TestRuntime>;
pub type OracleError = pallet_oracle::Error<TestRuntime>;

pub fn last_event() -> OracleEvent {
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
