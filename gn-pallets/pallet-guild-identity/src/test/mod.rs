mod address;
mod authorize;
mod identity;
mod register;

use crate::mock::*;
use frame_support::dispatch::DispatchError;
use frame_support::{assert_noop, assert_ok};

type IdentityEvent = pallet_guild_identity::Event<TestRuntime>;
type IdentityError = pallet_guild_identity::Error<TestRuntime>;
type OracleError = pallet_oracle::Error<TestRuntime>;

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
fn signer_checks() {
    new_test_ext().execute_with(|| {
        let failing_transactions = vec![
            <GuildIdentity>::register(RuntimeOrigin::none()),
            <GuildIdentity>::register(RuntimeOrigin::root()),
            <GuildIdentity>::deregister(RuntimeOrigin::none()),
            <GuildIdentity>::deregister(RuntimeOrigin::root()),
            <GuildIdentity>::authorize(RuntimeOrigin::none(), [0u8; 32], false),
            <GuildIdentity>::authorize(RuntimeOrigin::root(), [0u8; 32], true),
            <GuildIdentity>::link_address(RuntimeOrigin::none(), 0, [0u8; 8], [0u8; 65]),
            <GuildIdentity>::link_address(RuntimeOrigin::root(), 0, [0u8; 8], [0u8; 65]),
            <GuildIdentity>::unlink_address(RuntimeOrigin::none(), [0u8; 8], 0),
            <GuildIdentity>::unlink_address(RuntimeOrigin::root(), [0u8; 8], 0),
            <GuildIdentity>::link_identity(RuntimeOrigin::none(), [0u8; 8], [0u8; 32]),
            <GuildIdentity>::link_identity(RuntimeOrigin::root(), [0u8; 8], [0u8; 32]),
            <GuildIdentity>::unlink_identity(RuntimeOrigin::none(), [0u8; 8]),
            <GuildIdentity>::unlink_identity(RuntimeOrigin::root(), [0u8; 8]),
        ];

        for tx in failing_transactions {
            assert_noop!(tx, DispatchError::BadOrigin);
        }
    });
}
