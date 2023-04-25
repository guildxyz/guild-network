mod address;
mod authorize;
mod register;

use crate::mock::*;
use frame_support::dispatch::DispatchError;
use frame_support::{assert_noop, assert_ok};

type IdentityEvent = pallet_guild_identity::Event<TestRuntime>;
type IdentityError = pallet_guild_identity::Error<TestRuntime>;

fn last_event() -> IdentityEvent {
    System::events()
        .into_iter()
        .filter_map(|e| {
            if let RuntimeEvent::GuildIdentity(inner) = e.event {
                Some(inner)
            } else {
                None
            }
        })
        .last()
        .unwrap()
}

#[test]
fn invalid_transactions_fail() {
    new_test_ext().execute_with(|| {
        let failing_transactions = vec![
            (
                <GuildIdentity>::register(RuntimeOrigin::none()),
                DispatchError::BadOrigin,
            ),
            (
                <GuildIdentity>::register(RuntimeOrigin::root()),
                DispatchError::BadOrigin,
            ),
            (
                <GuildIdentity>::deregister(RuntimeOrigin::none()),
                DispatchError::BadOrigin,
            ),
            (
                <GuildIdentity>::deregister(RuntimeOrigin::root()),
                DispatchError::BadOrigin,
            ),
            (
                <GuildIdentity>::authorize(RuntimeOrigin::none(), [0u8; 32], false),
                DispatchError::BadOrigin,
            ),
            (
                <GuildIdentity>::authorize(RuntimeOrigin::root(), [0u8; 32], true),
                DispatchError::BadOrigin,
            ),
            (
                <GuildIdentity>::link_address(RuntimeOrigin::none(), 0, [0u8; 8], [0u8; 65]),
                DispatchError::BadOrigin,
            ),
            (
                <GuildIdentity>::link_address(RuntimeOrigin::root(), 0, [0u8; 8], [0u8; 65]),
                DispatchError::BadOrigin,
            ),
            (
                <GuildIdentity>::unlink_address(RuntimeOrigin::none(), [0u8; 8], 0),
                DispatchError::BadOrigin,
            ),
            (
                <GuildIdentity>::unlink_address(RuntimeOrigin::root(), [0u8; 8], 0),
                DispatchError::BadOrigin,
            ),
            (
                <GuildIdentity>::link_identity(RuntimeOrigin::none(), [0u8; 8], [0u8; 32]),
                DispatchError::BadOrigin,
            ),
            (
                <GuildIdentity>::link_identity(RuntimeOrigin::root(), [0u8; 8], [0u8; 32]),
                DispatchError::BadOrigin,
            ),
            (
                <GuildIdentity>::unlink_identity(RuntimeOrigin::none(), [0u8; 8]),
                DispatchError::BadOrigin,
            ),
            (
                <GuildIdentity>::unlink_identity(RuntimeOrigin::root(), [0u8; 8]),
                DispatchError::BadOrigin,
            ),
        ];

        for (tx, error) in failing_transactions {
            assert_noop!(tx, error);
        }
    });
}
