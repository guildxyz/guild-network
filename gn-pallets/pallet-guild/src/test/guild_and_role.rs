use super::*;
use gn_common::filter::{allowlist_filter, Filter, Logic as FilterLogic};

#[test]
fn guild_creation() {
    new_test_ext().execute_with(|| {
        init_chain();
        let signer = 4;
        let guild_name = [99u8; 32];
        let max_serialized_len =
            <TestRuntime as pallet_guild::Config>::MaxSerializedLen::get() as usize;

        dummy_guild(signer, guild_name);

        let failing_transactions = vec![
            (
                <Guild>::create_guild(RuntimeOrigin::none(), guild_name, vec![]),
                "BadOrigin",
            ),
            (
                <Guild>::create_guild(RuntimeOrigin::root(), guild_name, vec![]),
                "BadOrigin",
            ),
            (
                <Guild>::create_guild(RuntimeOrigin::signed(signer), guild_name, vec![]),
                "GuildAlreadyExists",
            ),
            (
                <Guild>::create_guild(
                    RuntimeOrigin::signed(signer),
                    [0u8; 32],
                    vec![0u8; max_serialized_len + 1],
                ),
                "MaxSerializedLenExceeded",
            ),
        ];

        for (tx, raw_error_msg) in failing_transactions {
            assert_eq!(error_msg(tx.unwrap_err()), raw_error_msg);
        }
    });
}

#[test]
fn guild_with_free_roles() {
    new_test_ext().execute_with(|| {
        init_chain();
        let signer = 1;
        let guild_name = [11u8; 32];
        let mut role_name = [22u8; 32];

        let other_signer = 2;
        let other_guild_name = [33u8; 32];
        let other_role_name = [1u8; 32];

        dummy_guild(signer, guild_name);
        let mut role_names = Vec::new();
        // successfully add free roles
        for i in 0..<TestRuntime as pallet_guild::Config>::MaxRolesPerGuild::get() as u8 {
            role_name[0] = i;
            role_names.push(role_name);
            <Guild>::create_free_role(RuntimeOrigin::signed(signer), guild_name, role_name)
                .unwrap();
            assert_eq!(
                last_event(),
                GuildEvent::RoleCreated(signer, guild_name, role_name)
            );
            let guild_id = <Guild>::guild_id(guild_name).unwrap();
            let guild = <Guild>::guild(guild_id).unwrap();
            assert_eq!(guild.name, guild_name);
            assert_eq!(guild.owner, signer);
            assert_eq!(guild.metadata, METADATA);
            assert_eq!(&guild.roles, &role_names);

            let role_id = <Guild>::role_id(guild_id, role_name).unwrap();
            let role = <Guild>::role(role_id).unwrap();
            assert!(role.filter.is_none());
            assert!(role.requirements.is_none());
        }

        let failing_transactions = vec![
            (
                <Guild>::create_free_role(RuntimeOrigin::none(), guild_name, role_name),
                "BadOrigin",
            ),
            (
                <Guild>::create_free_role(RuntimeOrigin::root(), other_guild_name, other_role_name),
                "BadOrigin",
            ),
            (
                <Guild>::create_free_role(
                    RuntimeOrigin::signed(signer),
                    other_guild_name,
                    other_role_name,
                ),
                "GuildDoesNotExist",
            ),
            (
                <Guild>::create_free_role(RuntimeOrigin::signed(signer), guild_name, role_name),
                "RoleAlreadyExists",
            ),
            (
                <Guild>::create_free_role(
                    RuntimeOrigin::signed(other_signer),
                    guild_name,
                    other_role_name,
                ),
                "BadOrigin",
            ),
            (
                <Guild>::create_free_role(
                    RuntimeOrigin::signed(signer),
                    guild_name,
                    other_role_name,
                ),
                "MaxRolesPerGuildExceeded",
            ),
        ];

        for (tx, raw_error_msg) in failing_transactions {
            assert_eq!(error_msg(tx.unwrap_err()), raw_error_msg);
        }
    });
}

#[test]
fn guild_with_allowlist_filter() {
    let allowlist_0 = vec![
        Identity::Address20([0u8; 20]),
        Identity::Address20([1u8; 20]),
    ];
    let allowlist_1 = vec![
        Identity::Address32([1u8; 32]),
        Identity::Address32([2u8; 32]),
        Identity::Address32([3u8; 32]),
        Identity::Address32([4u8; 32]),
    ];
    let mut ext = new_test_ext();
    let mut role_id_0 = Default::default();
    let mut role_id_1 = Default::default();

    ext.execute_with(|| {
        init_chain();
        let signer = 1;
        let guild_name = [11u8; 32];
        let role_name_0 = [0u8; 32];
        let role_name_1 = [1u8; 32];
        let filter_logic_0 = FilterLogic::And;
        let filter_logic_1 = FilterLogic::Or;

        dummy_guild(signer, guild_name);

        let failing_transactions = vec![
            (
                <Guild>::create_role_with_allowlist(
                    RuntimeOrigin::signed(signer),
                    guild_name,
                    role_name_0,
                    vec![],
                    FilterLogic::And,
                    None,
                ),
                "InvalidAllowlistLen",
            ),
            (
                <Guild>::create_role_with_allowlist(
                    RuntimeOrigin::signed(signer),
                    guild_name,
                    role_name_0,
                    vec![
                        Identity::Address20([0u8; 20]);
                        <TestRuntime as pallet_guild::Config>::MaxAllowlistLen::get() as usize + 1
                    ],
                    FilterLogic::And,
                    None,
                ),
                "InvalidAllowlistLen",
            ),
        ];
        for (tx, raw_error_msg) in failing_transactions {
            assert_eq!(error_msg(tx.unwrap_err()), raw_error_msg);
        }

        <Guild>::create_role_with_allowlist(
            RuntimeOrigin::signed(signer),
            guild_name,
            role_name_0,
            allowlist_0.clone(),
            filter_logic_0,
            None,
        )
        .unwrap();

        let filter_0 = allowlist_filter::<Keccak256>(&allowlist_0, filter_logic_0);

        <Guild>::create_role_with_allowlist(
            RuntimeOrigin::signed(signer),
            guild_name,
            role_name_1,
            allowlist_1.clone(),
            filter_logic_1,
            Some((vec![], vec![])),
        )
        .unwrap();

        let filter_1 = allowlist_filter::<Keccak256>(&allowlist_1, filter_logic_1);

        let guild_id = <Guild>::guild_id(guild_name).unwrap();
        let guild = <Guild>::guild(guild_id).unwrap();
        assert_eq!(guild.name, guild_name);
        assert_eq!(guild.owner, signer);
        assert_eq!(guild.metadata, METADATA);
        assert_eq!(guild.roles, &[role_name_0, role_name_1]);
        role_id_0 = <Guild>::role_id(guild_id, role_name_0).unwrap();
        role_id_1 = <Guild>::role_id(guild_id, role_name_1).unwrap();
        let role_0 = <Guild>::role(role_id_0).unwrap();
        let role_1 = <Guild>::role(role_id_1).unwrap();
        assert_eq!(role_0.filter, Some(filter_0));
        assert!(role_0.requirements.is_none());
        assert_eq!(role_1.filter, Some(filter_1));
        assert!(role_1.requirements.is_some());
    });
    // check offchain storage
    ext.persist_offchain_overlay();
    let offchain_db = ext.offchain_db();
    assert_ne!(role_id_0, Default::default());
    assert_ne!(role_id_1, Default::default());

    assert_eq!(
        offchain_db.get(&gn_common::offchain_allowlist_key(role_id_0.as_ref())),
        Some(allowlist_0.encode())
    );
    assert_eq!(
        offchain_db.get(&gn_common::offchain_allowlist_key(role_id_1.as_ref())),
        Some(allowlist_1.encode())
    );
}

/*
#[test]
fn advanced_checks() {
    new_test_ext().execute_with(|| {
        init_chain();
        let signer = 1;
        let guild_name = [0u8; 32];
        new_guild(signer, guild_name);

        let register_payload = RequestData::Register {
            identity_with_auth: IdentityWithAuth::Other(Identity::Other([0u8; 64]), [0u8; 64]),
            index: 0,
        };
        let reqcheck_payload = RequestData::ReqCheck {
            account: signer,
            guild: guild_name,
            role: ROLE_1,
        };

        // register oracle operator
        <Oracle>::register_operator(RuntimeOrigin::signed(signer)).unwrap();

        // register user
        <Guild>::register(RuntimeOrigin::signed(signer), register_payload.clone()).unwrap();
        let mut request_id = 0;
        let request = <Oracle>::request(request_id).unwrap();
        assert_eq!(request.requester, signer);
        assert_eq!(request.operator, signer);
        let request_data = Request::<AccountId>::decode(&mut request.data.as_slice()).unwrap();
        assert_eq!(request_data.data, register_payload);

        <Oracle>::callback(
            RuntimeOrigin::signed(signer),
            request_id,
            vec![u8::from(true)],
        )
        .unwrap();

        // someone else tries to assign a role to a registered user
        let error =
            <Guild>::manage_role(RuntimeOrigin::signed(signer + 1), reqcheck_payload.clone())
                .unwrap_err();
        assert_eq!(error_msg(error), "BadOrigin");

        // assign role to registered user
        <Guild>::manage_role(RuntimeOrigin::signed(signer), reqcheck_payload.clone()).unwrap();
        request_id += 1;
        let request = <Oracle>::request(request_id).unwrap();
        assert_eq!(request.requester, signer);
        assert_eq!(request.operator, signer);
        let request_data = Request::<AccountId>::decode(&mut request.data.as_slice()).unwrap();
        assert_eq!(request_data.data, reqcheck_payload);
        <Oracle>::callback(
            RuntimeOrigin::signed(signer),
            request_id,
            vec![u8::from(true)],
        )
        .unwrap();

        // strip role request successfully submitted by any signer
        <Guild>::manage_role(RuntimeOrigin::signed(signer + 1), reqcheck_payload.clone()).unwrap();
        request_id += 1;
        let request = <Oracle>::request(request_id).unwrap();
        assert_eq!(request.requester, signer + 1);
        assert_eq!(request_data.data, reqcheck_payload);

        // strip role request immediately executed if the signer is the same
        // without any oracle involvement
        <Guild>::manage_role(RuntimeOrigin::signed(signer), reqcheck_payload).unwrap();
        request_id += 1;
        assert!(<Oracle>::request(request_id).is_none());
    });
}

#[test]
#[rustfmt::skip]
fn storage_checks() {
    new_test_ext().execute_with(|| {
        init_chain();
        // setup two guilds
        let signer_1 = 1;
        let signer_2 = 2;
        let guild_1 = [1u8; 32];
        let guild_2 = [2u8; 32];
        new_guild(signer_1, guild_1);
        new_guild(signer_2, guild_2);

        let guild_id = <Guild>::guild_id(guild_1).unwrap();
        let g1r1_id = <Guild>::role_id(guild_id, ROLE_1).unwrap();
        let g1r2_id = <Guild>::role_id(guild_id, ROLE_2).unwrap();
        let g1r3_id = <Guild>::role_id(guild_id, ROLE_3).unwrap();
        let guild_id = <Guild>::guild_id(guild_2).unwrap();
        let g2r1_id = <Guild>::role_id(guild_id, ROLE_1).unwrap();
        let g2r2_id = <Guild>::role_id(guild_id, ROLE_2).unwrap();
        let g2r3_id = <Guild>::role_id(guild_id, ROLE_3).unwrap();

        let index = 0;
        let mut request_id = 0;

        // register a single operator
        <Oracle>::register_operator(RuntimeOrigin::signed(signer_1)).unwrap();
        // register both users to guild
        <Guild>::register(RuntimeOrigin::signed(signer_1), RequestData::Register {
            identity_with_auth: IdentityWithAuth::Other(Identity::Other([0u8; 64]), [0u8; 64]),
            index,
        }).unwrap();
        <Guild>::register(RuntimeOrigin::signed(signer_2), RequestData::Register {
            identity_with_auth: IdentityWithAuth::Other(Identity::Other([0u8; 64]), [0u8; 64]),
            index,
        }).unwrap();

        // both register requests are accepted
        <Oracle>::callback(RuntimeOrigin::signed(signer_1), request_id, vec![u8::from(true)]).unwrap();
        request_id += 1;
        <Oracle>::callback(RuntimeOrigin::signed(signer_1), request_id, vec![u8::from(true)]).unwrap();
        request_id += 1;
        assert!(<Guild>::user_data(signer_1, index).is_some());
        assert!(<Guild>::user_data(signer_2, index).is_some());

        // assign some roles to signer_1
        <Guild>::manage_role(RuntimeOrigin::signed(signer_1), RequestData::ReqCheck { account: signer_1, guild: guild_1, role: ROLE_1 }).unwrap();
        <Guild>::manage_role(RuntimeOrigin::signed(signer_1), RequestData::ReqCheck { account: signer_1, guild: guild_1, role: ROLE_2 }).unwrap();
        <Guild>::manage_role(RuntimeOrigin::signed(signer_1), RequestData::ReqCheck { account: signer_1, guild: guild_1, role: ROLE_3 }).unwrap();
        <Guild>::manage_role(RuntimeOrigin::signed(signer_1), RequestData::ReqCheck { account: signer_1, guild: guild_2, role: ROLE_1 }).unwrap();
        // assign some roles to signer_2
        <Guild>::manage_role(RuntimeOrigin::signed(signer_2), RequestData::ReqCheck { account: signer_2, guild: guild_1, role: ROLE_3 }).unwrap();
        <Guild>::manage_role(RuntimeOrigin::signed(signer_2), RequestData::ReqCheck { account: signer_2, guild: guild_2, role: ROLE_3 }).unwrap();
        // let in signer_1 to all roles except one
        <Oracle>::callback(RuntimeOrigin::signed(signer_1), request_id, vec![u8::from(true)]).unwrap();
        request_id += 1;
        <Oracle>::callback(RuntimeOrigin::signed(signer_1), request_id, vec![u8::from(true)]).unwrap();
        request_id += 1;
        <Oracle>::callback(RuntimeOrigin::signed(signer_1), request_id, vec![u8::from(true)]).unwrap();
        request_id += 1;
        assert_eq!(last_event(), GuildEvent::RoleAssigned(signer_1, guild_1, ROLE_3));
        let error = <Oracle>::callback(RuntimeOrigin::signed(signer_1), request_id, vec![u8::from(false)]).unwrap_err();
        assert_eq!(error_msg(error), "AccessDenied");
        request_id += 1;
        <Oracle>::callback(RuntimeOrigin::signed(signer_1), request_id, vec![u8::from(true)]).unwrap();
        request_id += 1;
        <Oracle>::callback(RuntimeOrigin::signed(signer_1), request_id, vec![u8::from(true)]).unwrap();
        request_id += 1;
        assert_eq!(last_event(), GuildEvent::RoleAssigned(signer_2, guild_2, ROLE_3));
        // check that all roles were properly assigned in storage
        assert!(<Guild>::member(g1r1_id, signer_1).is_some());
        assert!(<Guild>::member(g1r2_id, signer_1).is_some());
        assert!(<Guild>::member(g1r3_id, signer_1).is_some());
        assert!(<Guild>::member(g2r1_id, signer_1).is_none());
        assert!(<Guild>::member(g2r2_id, signer_1).is_none());
        assert!(<Guild>::member(g2r3_id, signer_1).is_none());

        assert!(<Guild>::member(g1r1_id, signer_2).is_none());
        assert!(<Guild>::member(g1r2_id, signer_2).is_none());
        assert!(<Guild>::member(g1r3_id, signer_2).is_some());
        assert!(<Guild>::member(g2r1_id, signer_2).is_none());
        assert!(<Guild>::member(g2r2_id, signer_2).is_none());
        assert!(<Guild>::member(g2r3_id, signer_2).is_some());
        // leave guilds voluntarily (check that no oracle request was sent)
        <Guild>::manage_role(RuntimeOrigin::signed(signer_1), RequestData::ReqCheck { account: signer_1, guild: guild_1, role: ROLE_1 }).unwrap();
        assert!(<Oracle>::request(request_id).is_none());
        assert!(<Guild>::member(g1r1_id, signer_1).is_none());
        <Guild>::manage_role(RuntimeOrigin::signed(signer_1), RequestData::ReqCheck { account: signer_1, guild: guild_1, role: ROLE_2 }).unwrap();
        assert!(<Oracle>::request(request_id).is_none());
        assert!(<Guild>::member(g1r2_id, signer_1).is_none());
        assert_eq!(last_event(), GuildEvent::RoleStripped(signer_1, guild_1, ROLE_2));
        // request a role check on another registered user
        <Guild>::manage_role(RuntimeOrigin::signed(signer_1), RequestData::ReqCheck { account: signer_2, guild: guild_1, role: ROLE_3 }).unwrap();
        <Guild>::manage_role(RuntimeOrigin::signed(signer_1), RequestData::ReqCheck { account: signer_2, guild: guild_2, role: ROLE_3 }).unwrap();
        // one request passes, but the other fails
        <Oracle>::callback(RuntimeOrigin::signed(signer_1), request_id, vec![u8::from(false)]).unwrap();
        request_id += 1;
        <Oracle>::callback(RuntimeOrigin::signed(signer_1), request_id, vec![u8::from(true)]).unwrap();
        request_id += 1;
        assert_eq!(last_event(), GuildEvent::RoleStripped(signer_2, guild_1, ROLE_3)); // g1r3 is stripped
        // check that signer 2 is stripped of guild1-role3, but kept guild2-role3
        assert!(<Guild>::member(g1r3_id, signer_2).is_none());
        assert!(<Guild>::member(g2r3_id, signer_2).is_some());
        // you cannot assign a role to another user
        let error = <Guild>::manage_role(RuntimeOrigin::signed(signer_2), RequestData::ReqCheck { account: signer_1, guild: guild_1, role: ROLE_1 }).unwrap_err();
        assert_eq!(error_msg(error), "BadOrigin");
        // request check on other user that still passes
        <Guild>::manage_role(RuntimeOrigin::signed(signer_2), RequestData::ReqCheck { account: signer_1, guild: guild_1, role: ROLE_3 }).unwrap();
        <Oracle>::callback(RuntimeOrigin::signed(signer_1), request_id, vec![u8::from(true)]).unwrap();
        request_id += 1;
        // fist user re-joins a previously left role
        <Guild>::manage_role(RuntimeOrigin::signed(signer_1), RequestData::ReqCheck { account: signer_1, guild: guild_1, role: ROLE_1 }).unwrap();
        <Oracle>::callback(RuntimeOrigin::signed(signer_1), request_id, vec![u8::from(true)]).unwrap();
        assert_eq!(last_event(), GuildEvent::RoleAssigned(signer_1, guild_1, ROLE_1));

        // check that all roles were properly assigned in storage
        assert!(<Guild>::member(g1r1_id, signer_1).is_some());
        assert!(<Guild>::member(g1r2_id, signer_1).is_none());
        assert!(<Guild>::member(g1r3_id, signer_1).is_some());
        assert!(<Guild>::member(g2r1_id, signer_1).is_none());
        assert!(<Guild>::member(g2r2_id, signer_1).is_none());
        assert!(<Guild>::member(g2r3_id, signer_1).is_none());

        assert!(<Guild>::member(g1r1_id, signer_2).is_none());
        assert!(<Guild>::member(g1r2_id, signer_2).is_none());
        assert!(<Guild>::member(g1r3_id, signer_2).is_none());
        assert!(<Guild>::member(g2r1_id, signer_2).is_none());
        assert!(<Guild>::member(g2r2_id, signer_2).is_none());
        assert!(<Guild>::member(g2r3_id, signer_2).is_some());
    });
}
*/
