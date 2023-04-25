use super::*;

#[test]
fn link_and_unlink_identities() {
    new_test_ext().execute_with(|| {
        let operator: <TestRuntime as frame_system::Config>::AccountId = 10;
        let account: <TestRuntime as frame_system::Config>::AccountId = 99;
        let prefix_0 = [0u8; 8];
        let prefix_1 = [1u8; 8];
        let prefix_2 = [3u8; 8];
        let prefix_3 = [3u8; 8];
        let identity_0 = [0u8; 32];
        let identity_1 = [1u8; 32];
        let identity_2 = [2u8; 32];
        let mut request_id = 0;
        // register oracle operator
        assert_ok!(<Oracle>::register_operator(RuntimeOrigin::root(), operator));
        assert_ok!(<Oracle>::activate_operator(RuntimeOrigin::signed(operator)));
        // try link request without registering first
        assert_noop!(
            <GuildIdentity>::link_identity(RuntimeOrigin::signed(account), prefix_0, identity_0),
            IdentityError::AccountDoesNotExist
        );
        // register
        assert_ok!(<GuildIdentity>::register(RuntimeOrigin::signed(account)));
        // link identity request
        assert_ok!(<GuildIdentity>::link_identity(
            RuntimeOrigin::signed(account),
            prefix_0,
            identity_0
        ));
        // oracle callback
        assert_noop!(
            <GuildIdentity>::callback(RuntimeOrigin::signed(account), 0, true),
            OracleError::WrongOperator
        );
        assert_ok!(<GuildIdentity>::callback(
            RuntimeOrigin::signed(operator),
            request_id,
            true
        ));
        assert_eq!(
            last_event(),
            IdentityEvent::IdentityLinked(account, prefix_0, identity_0)
        );
        // try link to the same prefix
        assert_noop!(
            <GuildIdentity>::link_identity(RuntimeOrigin::signed(account), prefix_0, identity_1,),
            IdentityError::IdentityAlreadyLinked,
        );
        // send two more identity link requests
        assert_ok!(<GuildIdentity>::link_identity(
            RuntimeOrigin::signed(account),
            prefix_1,
            identity_1
        ));
        assert_ok!(<GuildIdentity>::link_identity(
            RuntimeOrigin::signed(account),
            prefix_2,
            identity_2
        ));
        // ensure that only one identity was linked so far
        assert_eq!(<GuildIdentity>::identities(account).unwrap().len(), 1);
        assert_eq!(
            <GuildIdentity>::identities(account).unwrap().get(&prefix_0),
            Some(&identity_0)
        );
        // send two oracle responses
        request_id += 1;
        assert_noop!(
            <GuildIdentity>::callback(RuntimeOrigin::signed(operator), request_id, false),
            IdentityError::IdentityCheckFailed
        );
        request_id += 1;
        assert_ok!(<GuildIdentity>::callback(
            RuntimeOrigin::signed(operator),
            request_id,
            true
        ));
        assert_eq!(<GuildIdentity>::identities(account).unwrap().len(), 2);
        // resend previous response
        request_id -= 1;
        assert_ok!(<GuildIdentity>::callback(
            RuntimeOrigin::signed(operator),
            request_id,
            true
        ));
        // check storage
        assert_eq!(<GuildIdentity>::identities(account).unwrap().len(), 3);
        assert_eq!(
            <GuildIdentity>::identities(account)
                .unwrap()
                .values()
                .copied()
                .collect::<Vec<_>>(),
            &[identity_0, identity_1, identity_2]
        );
        // unlink identity
        assert_ok!(<GuildIdentity>::unlink_identity(
            RuntimeOrigin::signed(account),
            prefix_1
        ));
        assert_eq!(last_event(), IdentityEvent::IdentityUnlinked(account, prefix_1, identity_1));
        assert_eq!(<GuildIdentity>::identities(account).unwrap().len(), 2);
        assert!(<GuildIdentity>::identities(account)
            .unwrap()
            .get(&prefix_1)
            .is_none());
        // link again
        assert_ok!(<GuildIdentity>::link_identity(
            RuntimeOrigin::signed(account),
            prefix_1,
            identity_1,
        ));
        request_id += 2;
        assert_ok!(<GuildIdentity>::callback(
            RuntimeOrigin::signed(operator),
            request_id,
            true
        ));
        assert_eq!(<GuildIdentity>::identities(account).unwrap().len(), 3);
        // invalid transactions
        let failing_transactions = vec![
            (
                <GuildIdentity>::link_identity(
                    RuntimeOrigin::signed(account),
                    prefix_3,
                    identity_2,
                ),
                IdentityError::MaxLinkedIdentitiesExceeded,
            ),
            (
                <GuildIdentity>::link_identity(RuntimeOrigin::signed(255), prefix_3, identity_2),
                IdentityError::AccountDoesNotExist,
            ),
            (
                <GuildIdentity>::unlink_identity(RuntimeOrigin::signed(account), [11u8; 8]),
                IdentityError::IdentityDoesNotExist,
            ),
            (
                <GuildIdentity>::unlink_identity(RuntimeOrigin::signed(255), prefix_0),
                IdentityError::AccountDoesNotExist,
            ),
            (
                <GuildIdentity>::callback(RuntimeOrigin::signed(operator), 123, true),
                IdentityError::InvalidOracleAnswer,
            ),
        ];

        for (tx, error) in failing_transactions {
            assert_noop!(tx, error);
        }
    });
}
