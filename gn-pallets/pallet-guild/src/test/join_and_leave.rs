use super::*;
use gn_common::merkle::Proof as MerkleProof;
use parity_scale_codec::Encode;

#[test]
fn join_and_leave_free_role() {
    new_test_ext().execute_with(|| {
        let owner = 0;
        let user = 1;
        let guild_name = [0u8; 32];
        let role_name = [0u8; 32];
        let invalid_name = [100u8; 32];

        dummy_guild(owner, guild_name);

        assert_ok!(<Guild>::create_free_role(
            RuntimeOrigin::signed(owner),
            guild_name,
            role_name
        ));

        let failing_transactions = vec![
            (
                <Guild>::join_free_role(RuntimeOrigin::signed(user), invalid_name, role_name),
                GuildError::GuildDoesNotExist,
            ),
            (
                <Guild>::join_free_role(RuntimeOrigin::signed(user), guild_name, invalid_name),
                GuildError::RoleDoesNotExist,
            ),
            (
                <Guild>::join_free_role(RuntimeOrigin::signed(user), guild_name, role_name),
                GuildError::UserNotRegistered,
            ),
        ];
        for (tx, error) in failing_transactions {
            assert_noop!(tx, error);
        }
        // register user
        assert_ok!(<GuildIdentity>::register(RuntimeOrigin::signed(user),));
        // invalid join requests
        let failing_transactions = vec![
            <Guild>::join_child_role(RuntimeOrigin::signed(user), guild_name, role_name),
            <Guild>::join_role_with_allowlist(
                RuntimeOrigin::signed(user),
                guild_name,
                role_name,
                MerkleProof::new(&[&[1], &[2]], 0),
            ),
            <Guild>::join_unfiltered_role(RuntimeOrigin::signed(user), guild_name, role_name),
        ];

        for tx in failing_transactions {
            assert_noop!(tx, GuildError::InvalidJoinRequest);
        }
        // join free role
        assert_ok!(<Guild>::join_free_role(
            RuntimeOrigin::signed(user),
            guild_name,
            role_name
        ));
        assert_eq!(
            last_event(),
            GuildEvent::RoleAssigned(user, guild_name, role_name)
        );
        let guild_id = <Guild>::guild_id(guild_name).unwrap();
        let role_id = <Guild>::role_id(guild_id, role_name).unwrap();
        assert_eq!(<Guild>::member(role_id, user), Some(true));
        // leave free role
        assert_ok!(<Guild>::leave(
            RuntimeOrigin::signed(user),
            guild_name,
            role_name
        ));
        assert_eq!(
            last_event(),
            GuildEvent::RoleStripped(user, guild_name, role_name)
        );
        assert!(<Guild>::member(role_id, user).is_none());
    });
}

#[test]
fn join_and_leave_role_with_allowlist() {
    let owner: <TestRuntime as frame_system::Config>::AccountId = 0;
    let user_1: <TestRuntime as frame_system::Config>::AccountId = 1;
    let user_2: <TestRuntime as frame_system::Config>::AccountId = 2;
    let guild_name = [0u8; 32];
    let role_name = [0u8; 32];
    let allowlist = vec![12, user_1, 33];
    let mut role_id = Default::default();
    let mut ext = new_test_ext();

    ext.execute_with(|| {
        // user 1 registers
        assert_ok!(<GuildIdentity>::register(RuntimeOrigin::signed(user_1)));
        // user 2 registers
        assert_ok!(<GuildIdentity>::register(RuntimeOrigin::signed(user_2),));
        // owner creates a new guild
        dummy_guild(owner, guild_name);
        // owner creates a new role with allowlist
        assert_ok!(<Guild>::create_role_with_allowlist(
            RuntimeOrigin::signed(owner),
            guild_name,
            role_name,
            allowlist.clone(),
            FilterLogic::And,
            None,
        ));
        let guild_id = <Guild>::guild_id(guild_name).unwrap();
        role_id = <Guild>::role_id(guild_id, role_name).unwrap();
    });

    ext.persist_offchain_overlay();
    let offchain_db = ext.offchain_db();
    assert_eq!(
        offchain_db.get(&gn_common::offchain_allowlist_key(role_id.as_ref())),
        Some(allowlist.encode())
    );

    let proof_0 = MerkleProof::new(&allowlist, 0);
    let proof_1 = MerkleProof::new(&allowlist, 1);

    ext.execute_with(|| {
        let failing_transactions = vec![
            (
                <Guild>::join_role_with_allowlist(
                    RuntimeOrigin::signed(user_1),
                    guild_name,
                    role_name,
                    proof_0,
                ),
                GuildError::AccessDenied,
            ),
            (
                <Guild>::join_role_with_allowlist(
                    RuntimeOrigin::signed(user_2),
                    guild_name,
                    role_name,
                    proof_1.clone(),
                ),
                GuildError::AccessDenied,
            ),
            (
                <Guild>::join_free_role(RuntimeOrigin::signed(user_2), guild_name, role_name),
                GuildError::InvalidJoinRequest,
            ),
            (
                <Guild>::join_child_role(RuntimeOrigin::signed(user_2), guild_name, role_name),
                GuildError::InvalidJoinRequest,
            ),
            (
                <Guild>::join_unfiltered_role(RuntimeOrigin::signed(user_2), guild_name, role_name),
                GuildError::InvalidJoinRequest,
            ),
        ];

        for (tx, error) in failing_transactions {
            assert_noop!(tx, error);
        }

        assert_ok!(<Guild>::join_role_with_allowlist(
            RuntimeOrigin::signed(user_1),
            guild_name,
            role_name,
            proof_1,
        ));
        assert_eq!(
            last_event(),
            GuildEvent::RoleAssigned(user_1, guild_name, role_name)
        );

        assert_eq!(<Guild>::member(role_id, user_1), Some(true));

        assert_ok!(<Guild>::leave(
            RuntimeOrigin::signed(user_1),
            guild_name,
            role_name
        ));
        assert_eq!(
            last_event(),
            GuildEvent::RoleStripped(user_1, guild_name, role_name)
        );
        assert!(<Guild>::member(role_id, user_1).is_none());
    });
}

#[test]
fn join_and_leave_role_with_filter() {
    let owner = 0;
    let user_1 = 1;
    let user_2 = 2;
    let g0 = [0u8; 32];
    let g1 = [1u8; 32];
    let g0r0 = [0u8; 32];
    let g0r1 = [1u8; 32];
    let g0r2 = [2u8; 32];
    let g1r0 = [10u8; 32];
    let g1r1 = [11u8; 32];
    let g1r2 = [12u8; 32];
    let filter_logic = FilterLogic::And;
    let filter_0 = GuildFilter {
        name: g0,
        role: Some(g0r0),
    };
    let filter_1 = GuildFilter {
        name: g0,
        role: None,
    };

    new_test_ext().execute_with(|| {
        dummy_guild(owner, g0);
        dummy_guild(owner, g1);
        assert_ok!(<Guild>::create_free_role(
            RuntimeOrigin::signed(owner),
            g0,
            g0r0
        ));
        assert_ok!(<Guild>::create_free_role(
            RuntimeOrigin::signed(owner),
            g0,
            g0r1
        ));
        assert_ok!(<Guild>::create_child_role(
            RuntimeOrigin::signed(owner),
            g0,
            g0r2,
            filter_0,
            filter_logic,
            None,
        ));
        assert_ok!(<Guild>::create_free_role(
            RuntimeOrigin::signed(owner),
            g1,
            g1r0
        ));
        assert_ok!(<Guild>::create_free_role(
            RuntimeOrigin::signed(owner),
            g1,
            g1r1
        ));
        assert_ok!(<Guild>::create_child_role(
            RuntimeOrigin::signed(owner),
            g1,
            g1r2,
            filter_1,
            filter_logic,
            None,
        ));

        assert_ok!(<GuildIdentity>::register(RuntimeOrigin::signed(user_1)));
        assert_ok!(<GuildIdentity>::register(RuntimeOrigin::signed(user_2)));

        let g0_id = <Guild>::guild_id(g0).unwrap();
        let g0r0_id = <Guild>::role_id(g0_id, g0r0).unwrap();
        let g0r1_id = <Guild>::role_id(g0_id, g0r1).unwrap();
        let g0r2_id = <Guild>::role_id(g0_id, g0r2).unwrap();
        let g1_id = <Guild>::guild_id(g1).unwrap();
        let g1r0_id = <Guild>::role_id(g1_id, g1r0).unwrap();
        let g1r1_id = <Guild>::role_id(g1_id, g1r1).unwrap();
        let g1r2_id = <Guild>::role_id(g1_id, g1r2).unwrap();

        // user 1 joins guild 0 role 0
        assert_ok!(<Guild>::join_free_role(
            RuntimeOrigin::signed(user_1),
            g0,
            g0r0,
        ));
        // user 2 joins guild 0 role 1
        assert_ok!(<Guild>::join_free_role(
            RuntimeOrigin::signed(user_2),
            g0,
            g0r1,
        ));
        // user 1 can join guild 0 role 2 because they joined guild 0 role 0
        assert_ok!(<Guild>::join_child_role(
            RuntimeOrigin::signed(user_1),
            g0,
            g0r2,
        ));
        // user 2 cannot join guild 0 role 2 because they haven't joined guild 0 role 0
        assert_noop!(
            <Guild>::join_child_role(RuntimeOrigin::signed(user_2), g0, g0r2),
            GuildError::AccessDenied
        );
        assert!(<Guild>::member(g0r0_id, user_1).is_some());
        assert!(<Guild>::member(g0r1_id, user_1).is_none());
        assert!(<Guild>::member(g0r2_id, user_1).is_some());
        assert!(<Guild>::member(g0r0_id, user_2).is_none());
        assert!(<Guild>::member(g0r1_id, user_2).is_some());
        assert!(<Guild>::member(g0r2_id, user_2).is_none());

        // user 1 leaves all roles in guild 0
        assert_ok!(<Guild>::leave(RuntimeOrigin::signed(user_1), g0, g0r0));
        assert_ok!(<Guild>::leave(RuntimeOrigin::signed(user_1), g0, g0r2));
        assert!(<Guild>::member(g0r0_id, user_1).is_none());
        assert!(<Guild>::member(g0r1_id, user_1).is_none());
        assert!(<Guild>::member(g0r2_id, user_1).is_none());
        // now only user 2 can join guild 1 role 2 because it
        // requires at least one joined role in guild 0
        assert_ok!(<Guild>::join_free_role(
            RuntimeOrigin::signed(user_1),
            g1,
            g1r0,
        ));
        assert_ok!(<Guild>::join_free_role(
            RuntimeOrigin::signed(user_1),
            g1,
            g1r1,
        ));
        assert_noop!(
            <Guild>::join_child_role(RuntimeOrigin::signed(user_1), g1, g1r2),
            GuildError::AccessDenied,
        );
        assert_ok!(<Guild>::join_child_role(
            RuntimeOrigin::signed(user_2),
            g1,
            g1r2,
        ));

        assert!(<Guild>::member(g0r0_id, user_1).is_none());
        assert!(<Guild>::member(g0r1_id, user_1).is_none());
        assert!(<Guild>::member(g0r2_id, user_1).is_none());
        assert!(<Guild>::member(g0r0_id, user_2).is_none());
        assert!(<Guild>::member(g0r1_id, user_2).is_some());
        assert!(<Guild>::member(g0r2_id, user_2).is_none());

        assert!(<Guild>::member(g1r0_id, user_1).is_some());
        assert!(<Guild>::member(g1r1_id, user_1).is_some());
        assert!(<Guild>::member(g1r2_id, user_1).is_none());
        assert!(<Guild>::member(g1r0_id, user_2).is_none());
        assert!(<Guild>::member(g1r1_id, user_2).is_none());
        assert!(<Guild>::member(g1r2_id, user_2).is_some());

        let failing_transactions = vec![
            <Guild>::join_free_role(RuntimeOrigin::signed(user_2), g1, g1r2),
            <Guild>::join_role_with_allowlist(
                RuntimeOrigin::signed(user_2),
                g1,
                g1r2,
                MerkleProof::new(&[&[1], &[2]], 0),
            ),
            <Guild>::join_unfiltered_role(RuntimeOrigin::signed(user_2), g1, g1r2),
        ];

        for tx in failing_transactions {
            assert_noop!(tx, GuildError::InvalidJoinRequest);
        }
    });
}

#[test]
fn join_and_leave_unfiltered_role() {
    let owner = 0;
    let operator = 1;
    let user = 2;
    let guild_name = [0u8; 32];
    let role_name = [1u8; 32];

    new_test_ext().execute_with(|| {
        let mut request_id = 0;

        // new guild with unfiltered role
        dummy_guild(owner, guild_name);
        assert_ok!(<Guild>::create_unfiltered_role(
            RuntimeOrigin::signed(owner),
            guild_name,
            role_name,
            (vec![], vec![]),
        ));

        let guild_id = <Guild>::guild_id(guild_name).unwrap();
        let role_id = <Guild>::role_id(guild_id, role_name).unwrap();

        // register oracle operator
        assert_ok!(<Oracle>::register_operator(RuntimeOrigin::root(), operator));
        assert_ok!(<Oracle>::activate_operator(RuntimeOrigin::signed(operator)));
        assert_ok!(<GuildIdentity>::register(RuntimeOrigin::signed(user),));

        // try to get a role that requires oracle check
        assert_ok!(<Guild>::join_unfiltered_role(
            RuntimeOrigin::signed(user),
            guild_name,
            role_name
        ));
        assert_ok!(<Guild>::callback(
            RuntimeOrigin::signed(operator),
            request_id,
            true,
        ));
        request_id += 1;
        assert_eq!(
            last_event(),
            GuildEvent::RoleAssigned(user, guild_name, role_name)
        );
        assert!(<Guild>::member(role_id, user).is_some());
        // owner requests an oracle check on user
        assert_ok!(<Guild>::request_access_check(
            RuntimeOrigin::signed(owner),
            user,
            guild_name,
            role_name
        ));
        // operator responds with true, so nothing happens
        assert_ok!(<Guild>::callback(
            RuntimeOrigin::signed(operator),
            request_id,
            true,
        ));
        request_id += 1;
        assert!(<Guild>::member(role_id, user).is_some());
        // owner requests another oracle check on user
        assert_ok!(<Guild>::request_access_check(
            RuntimeOrigin::signed(owner),
            user,
            guild_name,
            role_name
        ));
        // operator responds with false, so user is stripped of the role
        assert_ok!(<Guild>::callback(
            RuntimeOrigin::signed(operator),
            request_id,
            false,
        ));
        assert_eq!(
            last_event(),
            GuildEvent::RoleStripped(user, guild_name, role_name)
        );
        assert!(<Guild>::member(role_id, user).is_none());
    });
}

#[test]
fn role_with_filtered_requirements() {
    let owner = 0;
    let operator = 1;
    let user = 2;
    let guild_name = [0u8; 32];
    let role_name_0 = [1u8; 32];
    let role_name_1 = [2u8; 32];
    let role_name_2 = [3u8; 32];
    let filter = GuildFilter {
        name: guild_name,
        role: Some(role_name_0),
    };

    let filter_logic_1 = FilterLogic::And;
    let filter_logic_2 = FilterLogic::Or;

    new_test_ext().execute_with(|| {
        let mut request_id = 0;

        // create guild with three roles
        dummy_guild(owner, guild_name);
        assert_ok!(<Guild>::create_free_role(
            RuntimeOrigin::signed(owner),
            guild_name,
            role_name_0
        ));
        assert_ok!(<Guild>::create_child_role(
            RuntimeOrigin::signed(owner),
            guild_name,
            role_name_1,
            filter,
            filter_logic_1,
            Some((vec![], vec![])),
        ));
        assert_ok!(<Guild>::create_child_role(
            RuntimeOrigin::signed(owner),
            guild_name,
            role_name_2,
            filter,
            filter_logic_2,
            Some((vec![], vec![])),
        ));

        let guild_id = <Guild>::guild_id(guild_name).unwrap();
        let role_id_0 = <Guild>::role_id(guild_id, role_name_0).unwrap();
        let role_id_1 = <Guild>::role_id(guild_id, role_name_1).unwrap();
        let role_id_2 = <Guild>::role_id(guild_id, role_name_2).unwrap();

        // register oracle operator
        assert_ok!(<Oracle>::register_operator(RuntimeOrigin::root(), operator));
        assert_ok!(<Oracle>::activate_operator(RuntimeOrigin::signed(operator)));
        // register identity that requires oracle check
        assert_ok!(<GuildIdentity>::register(RuntimeOrigin::signed(user)));
        // owner also registers an identity
        assert_ok!(<GuildIdentity>::register(RuntimeOrigin::signed(owner),));
        // user joins the free role
        assert_ok!(<Guild>::join_free_role(
            RuntimeOrigin::signed(user),
            guild_name,
            role_name_0
        ));
        assert_eq!(
            last_event(),
            GuildEvent::RoleAssigned(user, guild_name, role_name_0)
        );
        assert!(<Guild>::member(role_id_0, user).is_some());

        // user joins role 1 with AND filter logic so an oracle request is
        // dispatched
        assert_ok!(<Guild>::join_child_role(
            RuntimeOrigin::signed(user),
            guild_name,
            role_name_1
        ));
        // user doesn't become a member until the oracle responds with true
        assert!(<Guild>::member(role_id_1, user).is_none());
        assert_ok!(<Guild>::callback(
            RuntimeOrigin::signed(operator),
            request_id,
            true,
        ));
        request_id += 1;
        assert_eq!(
            last_event(),
            GuildEvent::RoleAssigned(user, guild_name, role_name_1)
        );
        assert!(<Guild>::member(role_id_1, user).is_some());

        // user joins role 2 with OR filter logic so no oracle request
        // dispatched
        assert_ok!(<Guild>::join_child_role(
            RuntimeOrigin::signed(user),
            guild_name,
            role_name_2
        ));
        assert_eq!(
            last_event(),
            GuildEvent::RoleAssigned(user, guild_name, role_name_2)
        );
        assert!(<Guild>::member(role_id_2, user).is_some());

        // owner tries to join role 1 with AND filter logic, which fails, so no
        // oracle rquest is dispatched
        assert_noop!(
            <Guild>::join_child_role(RuntimeOrigin::signed(owner), guild_name, role_name_1),
            GuildError::AccessDenied
        );
        assert!(<Guild>::member(role_id_1, owner).is_none());
        // owner tries to join role 2 with OR filter logic, so an oracle
        // request is dispatched
        assert_ok!(<Guild>::join_child_role(
            RuntimeOrigin::signed(owner),
            guild_name,
            role_name_2
        ));
        assert!(<Guild>::member(role_id_2, owner).is_none());
        // owner joins role because, even though the filter failed,
        // the requirement check by the oracle passed
        assert_ok!(<Guild>::callback(
            RuntimeOrigin::signed(operator),
            request_id,
            true,
        ));
        request_id += 1;
        assert_eq!(
            last_event(),
            GuildEvent::RoleAssigned(owner, guild_name, role_name_2)
        );
        assert!(<Guild>::member(role_id_2, owner).is_some());

        // request invalid oracle checks
        let failing_transactions = vec![
            (
                <Guild>::request_access_check(
                    RuntimeOrigin::signed(user),
                    user,
                    guild_name,
                    role_name_0,
                ),
                DispatchError::BadOrigin,
            ),
            (
                <Guild>::request_access_check(
                    RuntimeOrigin::signed(operator),
                    user,
                    role_name_0,
                    role_name_1,
                ),
                GuildError::GuildDoesNotExist.into(),
            ),
            (
                <Guild>::request_access_check(
                    RuntimeOrigin::signed(operator),
                    user,
                    guild_name,
                    guild_name,
                ),
                GuildError::RoleDoesNotExist.into(),
            ),
            (
                <Guild>::request_access_check(
                    RuntimeOrigin::signed(user),
                    operator,
                    guild_name,
                    role_name_0,
                ),
                GuildError::UserNotJoined.into(),
            ),
            (
                <Guild>::request_access_check(
                    RuntimeOrigin::signed(operator),
                    user,
                    guild_name,
                    role_name_2,
                ),
                GuildError::InvalidOracleRequest.into(),
            ),
        ];
        for (tx, error) in failing_transactions {
            assert_noop!(tx, error);
        }

        // owner requests a check on user in role 1 (with AND filter logic)
        assert_ok!(<Guild>::request_access_check(
            RuntimeOrigin::signed(owner),
            user,
            guild_name,
            role_name_1
        ));
        assert!(<Guild>::member(role_id_1, user).is_some());
        // role 1 is stripped from user due to the failed oracle check
        assert_ok!(<Guild>::callback(
            RuntimeOrigin::signed(operator),
            request_id,
            false,
        ));
        assert_eq!(
            last_event(),
            GuildEvent::RoleStripped(user, guild_name, role_name_1)
        );
        assert!(<Guild>::member(role_id_1, user).is_none());

        // sudo remove user
        assert_ok!(<Guild>::sudo_remove(
            RuntimeOrigin::root(),
            user,
            guild_name,
            role_name_0
        ));
        assert_eq!(
            last_event(),
            GuildEvent::RoleStripped(user, guild_name, role_name_0)
        );
        assert!(<Guild>::member(role_id_0, user).is_none());
    });
}
