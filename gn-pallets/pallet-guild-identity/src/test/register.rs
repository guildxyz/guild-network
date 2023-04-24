use super::*;
use frame_support::BoundedBTreeMap;

#[test]
fn register_and_deregister() {
    new_test_ext().execute_with(|| {
        let account = 99;
        // register
        assert_ok!(<GuildIdentity>::register(RuntimeOrigin::signed(account)));
        assert_eq!(
            <GuildIdentity>::addresses(account),
            Some(BoundedBTreeMap::new())
        );
        assert_eq!(
            <GuildIdentity>::identities(account),
            Some(BoundedBTreeMap::new())
        );
        assert_eq!(
            <GuildIdentity>::authorities(account),
            Some([[0u8; 32], [0u8; 32]])
        );
        assert_eq!(last_event(), IdentityEvent::AccountRegistered(account));
        assert_noop!(
            <GuildIdentity>::register(RuntimeOrigin::signed(account)),
            IdentityError::AccountAlreadyExists
        );
        // deregister
        assert_ok!(<GuildIdentity>::deregister(RuntimeOrigin::signed(account)));
        assert!(<GuildIdentity>::addresses(account).is_none());
        assert!(<GuildIdentity>::identities(account).is_none());
        assert!(<GuildIdentity>::authorities(account).is_none());
        assert_eq!(last_event(), IdentityEvent::AccountDeregistered(account));
        assert_noop!(
            <GuildIdentity>::deregister(RuntimeOrigin::signed(account)),
            IdentityError::AccountDoesNotExist
        );
        // register again
        assert_ok!(<GuildIdentity>::register(RuntimeOrigin::signed(account)));
    });
}
