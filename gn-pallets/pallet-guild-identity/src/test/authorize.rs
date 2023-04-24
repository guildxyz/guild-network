use super::*;

#[test]
fn authorization() {
    new_test_ext().execute_with(|| {
        let account = 99;
        let authority_0 = [0u8; 32];
        let authority_1 = [1u8; 32];
        let authority_2 = [2u8; 32];
        let authority_3 = [3u8; 32];

        // cannot authorize without registering first
        assert_noop!(
            <GuildIdentity>::authorize(RuntimeOrigin::signed(account), authority_1, false),
            IdentityError::AccountDoesNotExist
        );
        // register and authorize
        assert_ok!(<GuildIdentity>::register(RuntimeOrigin::signed(account)));
        assert_ok!(<GuildIdentity>::authorize(
            RuntimeOrigin::signed(account),
            authority_1,
            false
        ));
        assert_eq!(
            <GuildIdentity>::authorities(account),
            Some([authority_1, authority_0])
        );
        assert_eq!(
            last_event(),
            IdentityEvent::Authorized(account, authority_1)
        );
        // try to register the same authority to the other index
        assert_noop!(
            <GuildIdentity>::authorize(RuntimeOrigin::signed(account), authority_1, true),
            IdentityError::AlreadyAuthorized
        );
        // register another authority
        assert_ok!(<GuildIdentity>::authorize(
            RuntimeOrigin::signed(account),
            authority_2,
            true
        ));
        assert_eq!(
            <GuildIdentity>::authorities(account),
            Some([authority_1, authority_2])
        );
        assert_eq!(
            last_event(),
            IdentityEvent::Authorized(account, authority_2)
        );
        assert_ok!(<GuildIdentity>::authorize(
            RuntimeOrigin::signed(account),
            authority_3,
            false
        ));
        assert_eq!(
            <GuildIdentity>::authorities(account),
            Some([authority_3, authority_2])
        );
        assert_eq!(
            last_event(),
            IdentityEvent::Authorized(account, authority_3)
        );
        assert_ok!(<GuildIdentity>::authorize(
            RuntimeOrigin::signed(account),
            authority_0,
            true
        ));
        assert_eq!(
            <GuildIdentity>::authorities(account),
            Some([authority_3, authority_0])
        );
        assert_eq!(
            last_event(),
            IdentityEvent::Authorized(account, authority_0)
        );
    });
}
