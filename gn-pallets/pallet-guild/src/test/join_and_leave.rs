use super::*;
use gn_common::filter::{Guild as GuildFilter, Logic as FilterLogic};
use gn_common::merkle::Proof as MerkleProof;

#[test]
fn join_and_leave_free_role() {
    new_test_ext().execute_with(|| {
        init_chain();
        let owner = 0;
        let user = 1;
        let guild_name = [0u8; 32];
        let role_name = [0u8; 32];
        let invalid_name = [100u8; 32];

        let (address, signature) = dummy_ecdsa_id_with_auth(user, [2u8; 32]);

        dummy_guild(owner, guild_name);

        <Guild>::create_free_role(RuntimeOrigin::signed(owner), guild_name, role_name).unwrap();

        let failing_transactions = vec![
            (
                <Guild>::join(RuntimeOrigin::none(), guild_name, role_name, None),
                "BadOrigin",
            ),
            (
                <Guild>::join(RuntimeOrigin::root(), guild_name, role_name, None),
                "BadOrigin",
            ),
            (
                <Guild>::join(RuntimeOrigin::signed(user), invalid_name, role_name, None),
                "GuildDoesNotExist",
            ),
            (
                <Guild>::join(RuntimeOrigin::signed(user), guild_name, invalid_name, None),
                "RoleDoesNotExist",
            ),
            (
                <Guild>::join(RuntimeOrigin::signed(user), guild_name, role_name, None),
                "UserNotRegistered",
            ),
        ];

        for (tx, raw_error_msg) in failing_transactions {
            assert_eq!(error_msg(tx.unwrap_err()), raw_error_msg);
        }

        <Guild>::register(
            RuntimeOrigin::signed(user),
            IdentityWithAuth::Ecdsa(address, signature),
            0,
        )
        .unwrap();
        <Guild>::join(RuntimeOrigin::signed(user), guild_name, role_name, None).unwrap();
        assert_eq!(
            last_event(),
            GuildEvent::RoleAssigned(user, guild_name, role_name)
        );

        let guild_id = <Guild>::guild_id(guild_name).unwrap();
        let role_id = <Guild>::role_id(guild_id, role_name).unwrap();
        assert_eq!(<Guild>::member(role_id, user), Some(true));

        <Guild>::leave(RuntimeOrigin::signed(user), guild_name, role_name).unwrap();
        assert_eq!(
            last_event(),
            GuildEvent::RoleStripped(user, guild_name, role_name)
        );
        assert!(<Guild>::member(role_id, user).is_none());
    });
}

#[test]
fn join_and_leave_role_with_allowlist() {
    let id_index = 0;
    let owner = 0;
    let user_1 = 1;
    let user_2 = 2;
    let guild_name = [0u8; 32];
    let role_name = [0u8; 32];
    let mut allowlist = vec![
        Identity::Address20([0u8; 20]),
        Identity::Address20([1u8; 20]),
        Identity::Address20([2u8; 20]),
    ];
    let mut role_id = Default::default();
    let mut ext = new_test_ext();

    ext.execute_with(|| {
        init_chain();
        // user 1 registers
        let (address, signature) = dummy_ecdsa_id_with_auth(user_1, [1u8; 32]);
        allowlist.push(address);
        <Guild>::register(
            RuntimeOrigin::signed(user_1),
            IdentityWithAuth::Ecdsa(address, signature),
            id_index,
        )
        .unwrap();
        assert_eq!(last_event(), GuildEvent::IdRegistered(user_1, id_index));

        // user 2 registers
        let (address, signature) = dummy_ecdsa_id_with_auth(user_2, [2u8; 32]);
        <Guild>::register(
            RuntimeOrigin::signed(user_2),
            IdentityWithAuth::Ecdsa(address, signature),
            id_index,
        )
        .unwrap();
        assert_eq!(last_event(), GuildEvent::IdRegistered(user_2, id_index));

        // owner creates a new guild
        dummy_guild(owner, guild_name);
        // owner creates a new role with allowlist
        <Guild>::create_role_with_allowlist(
            RuntimeOrigin::signed(owner),
            guild_name,
            role_name,
            allowlist.clone(),
            FilterLogic::And,
            None,
        )
        .unwrap();
        let guild_id = <Guild>::guild_id(guild_name).unwrap();
        role_id = <Guild>::role_id(guild_id, role_name).unwrap();
    });

    ext.persist_offchain_overlay();
    let offchain_db = ext.offchain_db();
    assert_eq!(
        offchain_db.get(&gn_common::offchain_allowlist_key(role_id.as_ref())),
        Some(allowlist.encode())
    );
    let leaf_index = allowlist.len() - 1;
    let proof = MerkleProof::new(&allowlist, leaf_index, id_index);

    let proof_with_invalid_path = MerkleProof {
        path: vec![],
        id_index,
    };

    let proof_with_invalid_id_index = MerkleProof {
        path: proof.path.clone(),
        id_index: id_index + 1,
    };

    ext.execute_with(|| {
        let failing_transactions = vec![
            (
                <Guild>::join(RuntimeOrigin::signed(user_1), guild_name, role_name, None),
                "MissingAllowlistProof",
            ),
            (
                <Guild>::join(
                    RuntimeOrigin::signed(user_1),
                    guild_name,
                    role_name,
                    Some(proof_with_invalid_path),
                ),
                "AccessDenied",
            ),
            (
                <Guild>::join(
                    RuntimeOrigin::signed(user_1),
                    guild_name,
                    role_name,
                    Some(proof_with_invalid_id_index),
                ),
                "IdNotRegistered",
            ),
            (
                <Guild>::join(
                    RuntimeOrigin::signed(user_2),
                    guild_name,
                    role_name,
                    Some(proof.clone()),
                ),
                "AccessDenied",
            ),
        ];

        for (tx, raw_error_msg) in failing_transactions {
            assert_eq!(error_msg(tx.unwrap_err()), raw_error_msg);
        }

        <Guild>::join(
            RuntimeOrigin::signed(user_1),
            guild_name,
            role_name,
            Some(proof),
        )
        .unwrap();
        assert_eq!(
            last_event(),
            GuildEvent::RoleAssigned(user_1, guild_name, role_name)
        );

        assert_eq!(<Guild>::member(role_id, user_1), Some(true));

        <Guild>::leave(RuntimeOrigin::signed(user_1), guild_name, role_name).unwrap();
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
        init_chain();
        dummy_guild(owner, g0);
        dummy_guild(owner, g1);
        <Guild>::create_free_role(RuntimeOrigin::signed(owner), g0, g0r0).unwrap();
        <Guild>::create_free_role(RuntimeOrigin::signed(owner), g0, g0r1).unwrap();
        <Guild>::create_child_role(
            RuntimeOrigin::signed(owner),
            g0,
            g0r2,
            filter_0,
            filter_logic,
            None,
        )
        .unwrap();
        <Guild>::create_free_role(RuntimeOrigin::signed(owner), g1, g1r0).unwrap();
        <Guild>::create_free_role(RuntimeOrigin::signed(owner), g1, g1r1).unwrap();
        <Guild>::create_child_role(
            RuntimeOrigin::signed(owner),
            g1,
            g1r2,
            filter_1,
            filter_logic,
            None,
        )
        .unwrap();

        let (address, signature) = dummy_ecdsa_id_with_auth(user_1, [1u8; 32]);
        <Guild>::register(
            RuntimeOrigin::signed(user_1),
            IdentityWithAuth::Ecdsa(address, signature),
            0,
        )
        .unwrap();
        let (address, signature) = dummy_ecdsa_id_with_auth(user_2, [2u8; 32]);
        <Guild>::register(
            RuntimeOrigin::signed(user_2),
            IdentityWithAuth::Ecdsa(address, signature),
            0,
        )
        .unwrap();

        let g0_id = <Guild>::guild_id(g0).unwrap();
        let g0r0_id = <Guild>::role_id(g0_id, g0r0).unwrap();
        let g0r1_id = <Guild>::role_id(g0_id, g0r1).unwrap();
        let g0r2_id = <Guild>::role_id(g0_id, g0r2).unwrap();
        let g1_id = <Guild>::guild_id(g1).unwrap();
        let g1r0_id = <Guild>::role_id(g1_id, g1r0).unwrap();
        let g1r1_id = <Guild>::role_id(g1_id, g1r1).unwrap();
        let g1r2_id = <Guild>::role_id(g1_id, g1r2).unwrap();

        // user 1 joins guild 0 role 0
        <Guild>::join(RuntimeOrigin::signed(user_1), g0, g0r0, None).unwrap();
        // user 2 joins guild 0 role 1
        <Guild>::join(RuntimeOrigin::signed(user_2), g0, g0r1, None).unwrap();
        // user 1 can join guild 0 role 2 because they joined guild 0 role 0
        <Guild>::join(RuntimeOrigin::signed(user_1), g0, g0r2, None).unwrap();
        // user 2 cannot join guild 0 role 2 because they haven't joined guild 0 role 0
        assert_eq!(
            error_msg(<Guild>::join(RuntimeOrigin::signed(user_2), g0, g0r2, None).unwrap_err()),
            "AccessDenied"
        );
        assert!(<Guild>::member(g0r0_id, user_1).is_some());
        assert!(<Guild>::member(g0r1_id, user_1).is_none());
        assert!(<Guild>::member(g0r2_id, user_1).is_some());
        assert!(<Guild>::member(g0r0_id, user_2).is_none());
        assert!(<Guild>::member(g0r1_id, user_2).is_some());
        assert!(<Guild>::member(g0r2_id, user_2).is_none());

        // user 1 leaves all roles in guild 0
        <Guild>::leave(RuntimeOrigin::signed(user_1), g0, g0r0).unwrap();
        <Guild>::leave(RuntimeOrigin::signed(user_1), g0, g0r2).unwrap();
        assert!(<Guild>::member(g0r0_id, user_1).is_none());
        assert!(<Guild>::member(g0r1_id, user_1).is_none());
        assert!(<Guild>::member(g0r2_id, user_1).is_none());
        // now only user 2 can join guild 1 role 2 because it
        // requires at least one joined role in guild 0
        <Guild>::join(RuntimeOrigin::signed(user_1), g1, g1r0, None).unwrap();
        <Guild>::join(RuntimeOrigin::signed(user_1), g1, g1r1, None).unwrap();
        assert_eq!(
            error_msg(<Guild>::join(RuntimeOrigin::signed(user_1), g1, g1r2, None).unwrap_err()),
            "AccessDenied"
        );
        <Guild>::join(RuntimeOrigin::signed(user_2), g1, g1r2, None).unwrap();

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
        init_chain();
        let mut request_id = 0;

        // new guild with unfiltered role
        dummy_guild(owner, guild_name);
        <Guild>::create_unfiltered_role(
            RuntimeOrigin::signed(owner),
            guild_name,
            role_name,
            (vec![], vec![]),
        )
        .unwrap();

        let guild_id = <Guild>::guild_id(guild_name).unwrap();
        let role_id = <Guild>::role_id(guild_id, role_name).unwrap();

        // register oracle operator
        <Oracle>::register_operator(RuntimeOrigin::signed(operator)).unwrap();
        // register identity that requires oracle check
        <Guild>::register(
            RuntimeOrigin::signed(user),
            IdentityWithAuth::Other(Identity::Other([0u8; 64]), [0u8; 64]),
            0,
        )
        .unwrap();

        <Oracle>::callback(
            RuntimeOrigin::signed(operator),
            request_id,
            vec![u8::from(true)],
        )
        .unwrap();

        assert_eq!(last_event(), GuildEvent::IdRegistered(user, 0));
        request_id += 1;

        // try to get a role that requires oracle check
        <Guild>::join(RuntimeOrigin::signed(user), guild_name, role_name, None).unwrap();
        <Oracle>::callback(
            RuntimeOrigin::signed(operator),
            request_id,
            vec![u8::from(true)],
        )
        .unwrap();
        request_id += 1;
        assert_eq!(
            last_event(),
            GuildEvent::RoleAssigned(user, guild_name, role_name)
        );
        assert!(<Guild>::member(role_id, user).is_some());
        // owner requests an oracle check on user
        <Guild>::request_oracle_check(RuntimeOrigin::signed(owner), user, guild_name, role_name)
            .unwrap();
        // operator responds with true, so nothing happens
        <Oracle>::callback(
            RuntimeOrigin::signed(operator),
            request_id,
            vec![u8::from(true)],
        )
        .unwrap();
        request_id += 1;
        assert!(<Guild>::member(role_id, user).is_some());
        // owner requests another oracle check on user
        <Guild>::request_oracle_check(RuntimeOrigin::signed(owner), user, guild_name, role_name)
            .unwrap();
        // operator responds with false, so user is stripped of the role
        <Oracle>::callback(
            RuntimeOrigin::signed(operator),
            request_id,
            vec![u8::from(false)],
        )
        .unwrap();
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
        init_chain();
        let mut request_id = 0;

        // create guild with three roles
        dummy_guild(owner, guild_name);
        <Guild>::create_free_role(RuntimeOrigin::signed(owner), guild_name, role_name_0).unwrap();
        <Guild>::create_child_role(
            RuntimeOrigin::signed(owner),
            guild_name,
            role_name_1,
            filter,
            filter_logic_1,
            Some((vec![], vec![])),
        )
        .unwrap();
        <Guild>::create_child_role(
            RuntimeOrigin::signed(owner),
            guild_name,
            role_name_2,
            filter,
            filter_logic_2,
            Some((vec![], vec![])),
        )
        .unwrap();

        let guild_id = <Guild>::guild_id(guild_name).unwrap();
        let role_id_0 = <Guild>::role_id(guild_id, role_name_0).unwrap();
        let role_id_1 = <Guild>::role_id(guild_id, role_name_1).unwrap();
        let role_id_2 = <Guild>::role_id(guild_id, role_name_2).unwrap();

        // register oracle operator
        <Oracle>::register_operator(RuntimeOrigin::signed(operator)).unwrap();

        // register identity that requires oracle check
        <Guild>::register(
            RuntimeOrigin::signed(user),
            IdentityWithAuth::Other(Identity::Other([0u8; 64]), [0u8; 64]),
            0,
        )
        .unwrap();

        <Oracle>::callback(
            RuntimeOrigin::signed(operator),
            request_id,
            vec![u8::from(true)],
        )
        .unwrap();
        request_id += 1;

        assert_eq!(last_event(), GuildEvent::IdRegistered(user, 0));
        // owner also registers an identity
        let (address, signature) = dummy_ecdsa_id_with_auth(owner, [2u8; 32]);
        <Guild>::register(
            RuntimeOrigin::signed(owner),
            IdentityWithAuth::Ecdsa(address, signature),
            0,
        )
        .unwrap();
        assert_eq!(last_event(), GuildEvent::IdRegistered(owner, 0));

        // user and owner both join the free role
        <Guild>::join(RuntimeOrigin::signed(user), guild_name, role_name_0, None).unwrap();
        assert_eq!(
            last_event(),
            GuildEvent::RoleAssigned(user, guild_name, role_name_0)
        );
        assert!(<Guild>::member(role_id_0, user).is_some());

        // user joins role 1 with AND filter logic so an oracle request is
        // dispatched
        <Guild>::join(RuntimeOrigin::signed(user), guild_name, role_name_1, None).unwrap();
        // user doesn't become a member until the oracle responds with true
        assert!(<Guild>::member(role_id_1, user).is_none());
        <Oracle>::callback(
            RuntimeOrigin::signed(operator),
            request_id,
            vec![u8::from(true)],
        )
        .unwrap();
        request_id += 1;
        assert_eq!(
            last_event(),
            GuildEvent::RoleAssigned(user, guild_name, role_name_1)
        );
        assert!(<Guild>::member(role_id_1, user).is_some());

        // user joins role 2 with OR filter logic so no oracle request
        // dispatched
        <Guild>::join(RuntimeOrigin::signed(user), guild_name, role_name_2, None).unwrap();
        assert_eq!(
            last_event(),
            GuildEvent::RoleAssigned(user, guild_name, role_name_2)
        );
        assert!(<Guild>::member(role_id_2, user).is_some());

        // owner tries to join role 1 with AND filter logic, which fails, so no
        // oracle rquest is dispatched
        assert_eq!(
            error_msg(
                <Guild>::join(RuntimeOrigin::signed(owner), guild_name, role_name_1, None)
                    .unwrap_err()
            ),
            "AccessDenied"
        );
        assert!(<Guild>::member(role_id_1, owner).is_none());
        // owner tries to join role 2 with OR filter logic, so an oracle
        // request is dispatched
        <Guild>::join(RuntimeOrigin::signed(owner), guild_name, role_name_2, None).unwrap();
        assert!(<Guild>::member(role_id_2, owner).is_none());
        // owner joins role because, even though the filter failed,
        // the requirement check by the oracle passed
        <Oracle>::callback(
            RuntimeOrigin::signed(operator),
            request_id,
            vec![u8::from(true)],
        )
        .unwrap();
        request_id += 1;
        assert_eq!(
            last_event(),
            GuildEvent::RoleAssigned(owner, guild_name, role_name_2)
        );
        assert!(<Guild>::member(role_id_2, owner).is_some());

        // request invalid oracle checks
        let failing_transactions = vec![
            (
                <Guild>::request_oracle_check(
                    RuntimeOrigin::signed(user),
                    user,
                    guild_name,
                    role_name_0,
                ),
                "BadOrigin",
            ),
            (
                <Guild>::request_oracle_check(
                    RuntimeOrigin::signed(operator),
                    owner,
                    guild_name,
                    role_name_1,
                ),
                "InvalidOracleRequest",
            ),
            (
                <Guild>::request_oracle_check(
                    RuntimeOrigin::signed(operator),
                    user,
                    role_name_0,
                    role_name_1,
                ),
                "GuildDoesNotExist",
            ),
            (
                <Guild>::request_oracle_check(
                    RuntimeOrigin::signed(operator),
                    user,
                    guild_name,
                    guild_name,
                ),
                "RoleDoesNotExist",
            ),
            (
                <Guild>::request_oracle_check(
                    RuntimeOrigin::signed(user),
                    operator,
                    guild_name,
                    role_name_0,
                ),
                "UserNotRegistered",
            ),
            (
                <Guild>::request_oracle_check(
                    RuntimeOrigin::signed(operator),
                    user,
                    guild_name,
                    role_name_2,
                ),
                "InvalidOracleRequest",
            ),
        ];
        for (tx, raw_error_msg) in failing_transactions {
            assert_eq!(error_msg(tx.unwrap_err()), raw_error_msg);
        }

        // owner requests a check on user in role 1 (with AND filter logic)
        <Guild>::request_oracle_check(RuntimeOrigin::signed(owner), user, guild_name, role_name_1)
            .unwrap();
        assert!(<Guild>::member(role_id_1, user).is_some());
        // role 1 is stripped from user due to the failed oracle check
        <Oracle>::callback(
            RuntimeOrigin::signed(operator),
            request_id,
            vec![u8::from(false)],
        )
        .unwrap();
        assert_eq!(
            last_event(),
            GuildEvent::RoleStripped(user, guild_name, role_name_1)
        );
        assert!(<Guild>::member(role_id_1, user).is_none());
    });
}
