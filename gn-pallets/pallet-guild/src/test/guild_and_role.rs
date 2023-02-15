use super::*;
use gn_common::filter::{allowlist_filter, Filter, Guild as GuildFilter, Logic as FilterLogic};

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
fn role_with_allowlist_filter() {
    let allowlist_0 = vec![
        Identity::Address20([0u8; 20]),
        Identity::Address20([1u8; 20]),
        Identity::Address20([2u8; 20]),
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
                    RuntimeOrigin::root(),
                    guild_name,
                    role_name_0,
                    vec![Identity::Address20([0u8; 20]); 1],
                    FilterLogic::And,
                    None,
                ),
                "BadOrigin",
            ),
            (
                <Guild>::create_role_with_allowlist(
                    RuntimeOrigin::none(),
                    guild_name,
                    role_name_0,
                    vec![Identity::Address20([0u8; 20]); 1],
                    FilterLogic::And,
                    None,
                ),
                "BadOrigin",
            ),
            (
                <Guild>::create_role_with_allowlist(
                    RuntimeOrigin::signed(signer + 1),
                    guild_name,
                    role_name_0,
                    vec![Identity::Address20([0u8; 20]); 1],
                    FilterLogic::And,
                    None,
                ),
                "BadOrigin",
            ),
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

        let guild_id = <Guild>::guild_id(guild_name).unwrap();

        <Guild>::create_role_with_allowlist(
            RuntimeOrigin::signed(signer),
            guild_name,
            role_name_0,
            allowlist_0.clone(),
            filter_logic_0,
            None,
        )
        .unwrap();

        role_id_0 = <Guild>::role_id(guild_id, role_name_0).unwrap();
        assert_eq!(
            last_event(),
            GuildEvent::AllowlistWritten(gn_common::offchain_allowlist_key(role_id_0.as_ref()))
        );

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
        role_id_1 = <Guild>::role_id(guild_id, role_name_1).unwrap();

        let filter_1 = allowlist_filter::<Keccak256>(&allowlist_1, filter_logic_1);

        let guild = <Guild>::guild(guild_id).unwrap();
        assert_eq!(guild.name, guild_name);
        assert_eq!(guild.owner, signer);
        assert_eq!(guild.metadata, METADATA);
        assert_eq!(guild.roles, &[role_name_0, role_name_1]);
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

#[test]
fn role_with_guild_filter() {
    new_test_ext().execute_with(|| {
        init_chain();
        let signer = 1;
        let guild_name_0 = [0u8; 32];
        let guild_name_1 = [1u8; 32];
        let guild_name_2 = [2u8; 32];
        let role_name_0 = [0u8; 32];
        let role_name_1 = [1u8; 32];
        let filter_logic_0 = FilterLogic::And;
        let filter_logic_1 = FilterLogic::Or;
        let filter_0 = GuildFilter {
            name: guild_name_1,
            role: None,
        };
        let filter_1 = GuildFilter {
            name: guild_name_0,
            role: Some(role_name_0),
        };

        dummy_guild(signer, guild_name_0);
        dummy_guild(signer, guild_name_1);
        <Guild>::create_free_role(RuntimeOrigin::signed(signer), guild_name_1, role_name_0)
            .unwrap();

        let failing_transactions = vec![
            (
                <Guild>::create_child_role(
                    RuntimeOrigin::none(),
                    guild_name_1,
                    role_name_0,
                    filter_0,
                    filter_logic_0,
                    None,
                ),
                "BadOrigin",
            ),
            (
                <Guild>::create_child_role(
                    RuntimeOrigin::root(),
                    guild_name_1,
                    role_name_0,
                    filter_0,
                    filter_logic_0,
                    None,
                ),
                "BadOrigin",
            ),
            (
                <Guild>::create_child_role(
                    RuntimeOrigin::signed(signer),
                    guild_name_2,
                    role_name_0,
                    filter_0,
                    filter_logic_0,
                    None,
                ),
                "GuildDoesNotExist",
            ),
            (
                <Guild>::create_child_role(
                    RuntimeOrigin::signed(signer),
                    guild_name_0,
                    role_name_1,
                    filter_1,
                    filter_logic_1,
                    None,
                ),
                "RoleDoesNotExist",
            ),
        ];
        for (tx, raw_error_msg) in failing_transactions {
            assert_eq!(error_msg(tx.unwrap_err()), raw_error_msg);
        }

        <Guild>::create_child_role(
            RuntimeOrigin::signed(signer),
            guild_name_0,
            role_name_0,
            filter_0,
            filter_logic_0,
            None,
        )
        .unwrap();
        assert_eq!(
            last_event(),
            GuildEvent::RoleCreated(signer, guild_name_0, role_name_0)
        );

        <Guild>::create_child_role(
            RuntimeOrigin::signed(signer),
            guild_name_0,
            role_name_1,
            filter_1,
            filter_logic_1,
            Some((vec![], vec![])),
        )
        .unwrap();

        assert_eq!(
            last_event(),
            GuildEvent::RoleCreated(signer, guild_name_0, role_name_1)
        );
        let guild_id = <Guild>::guild_id(guild_name_0).unwrap();
        let guild = <Guild>::guild(guild_id).unwrap();
        assert_eq!(guild.name, guild_name_0);
        assert_eq!(guild.owner, signer);
        assert_eq!(guild.metadata, METADATA);
        assert_eq!(&guild.roles, &[role_name_0, role_name_1]);

        let role_id = <Guild>::role_id(guild_id, role_name_0).unwrap();
        let role = <Guild>::role(role_id).unwrap();
        assert_eq!(role.filter, Some(Filter::Guild(filter_0, filter_logic_0)));
        assert!(role.requirements.is_none());

        let role_id = <Guild>::role_id(guild_id, role_name_1).unwrap();
        let role = <Guild>::role(role_id).unwrap();
        assert_eq!(role.filter, Some(Filter::Guild(filter_1, filter_logic_1)));
        assert!(role.requirements.is_some());
    });
}

#[test]
fn unfiltered_role() {
    new_test_ext().execute_with(|| {
        init_chain();
        let signer = 1;
        let guild_name = [0u8; 32];
        let role_name = [2u8; 32];
        let max_reqs_per_role =
            <TestRuntime as pallet_guild::Config>::MaxReqsPerRole::get() as usize;
        let max_serialized_len =
            <TestRuntime as pallet_guild::Config>::MaxSerializedLen::get() as usize;

        dummy_guild(signer, guild_name);

        let failing_transactions = vec![
            (
                <Guild>::create_unfiltered_role(
                    RuntimeOrigin::none(),
                    guild_name,
                    role_name,
                    (vec![], vec![]),
                ),
                "BadOrigin",
            ),
            (
                <Guild>::create_unfiltered_role(
                    RuntimeOrigin::root(),
                    guild_name,
                    role_name,
                    (vec![], vec![]),
                ),
                "BadOrigin",
            ),
            (
                <Guild>::create_unfiltered_role(
                    RuntimeOrigin::signed(signer + 1),
                    guild_name,
                    role_name,
                    (vec![], vec![]),
                ),
                "BadOrigin",
            ),
            (
                <Guild>::create_unfiltered_role(
                    RuntimeOrigin::signed(signer),
                    guild_name,
                    role_name,
                    (vec![vec![]; max_reqs_per_role + 1], vec![]),
                ),
                "MaxReqsPerRoleExceeded",
            ),
            (
                <Guild>::create_unfiltered_role(
                    RuntimeOrigin::signed(signer),
                    guild_name,
                    role_name,
                    (
                        vec![vec![]; max_reqs_per_role],
                        vec![0; max_serialized_len + 1],
                    ),
                ),
                "MaxSerializedLenExceeded",
            ),
            (
                <Guild>::create_unfiltered_role(
                    RuntimeOrigin::signed(signer),
                    guild_name,
                    role_name,
                    (
                        vec![vec![0; max_serialized_len + 1]; max_reqs_per_role],
                        vec![0; max_serialized_len],
                    ),
                ),
                "MaxSerializedLenExceeded",
            ),
        ];
        for (tx, raw_error_msg) in failing_transactions {
            assert_eq!(error_msg(tx.unwrap_err()), raw_error_msg);
        }
        let valid_requirements = (
            vec![vec![0; max_serialized_len]; max_reqs_per_role],
            vec![0; max_serialized_len],
        );
        <Guild>::create_unfiltered_role(
            RuntimeOrigin::signed(signer),
            guild_name,
            role_name,
            valid_requirements.clone(),
        )
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
        assert_eq!(&guild.roles, &[role_name]);

        let role_id = <Guild>::role_id(guild_id, role_name).unwrap();
        let role = <Guild>::role(role_id).unwrap();
        assert!(role.filter.is_none());
        assert_eq!(role.requirements, Some(valid_requirements))
    });
}
