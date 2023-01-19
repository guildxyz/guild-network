use super::*;

#[test]
fn basic_checks() {
    new_test_runtime().execute_with(|| {
        init_chain();
        let signer = 1;
        let guild_name = [0u8; 32];
        new_guild(signer, guild_name);

        let test_data = vec![
            (RequestData::Register(vec![]), "InvalidRequestData"),
            (
                RequestData::ReqCheck {
                    account: signer,
                    guild: [111; 32],
                    role: [255; 32],
                },
                "GuildDoesNotExist",
            ),
            (
                RequestData::ReqCheck {
                    account: signer,
                    guild: guild_name,
                    role: [255; 32],
                },
                "RoleDoesNotExist",
            ),
            (
                RequestData::ReqCheck {
                    account: signer,
                    guild: guild_name,
                    role: ROLE_1,
                },
                "UserNotRegistered",
            ),
        ];

        for (request, raw_error_msg) in test_data {
            let error = <Guild>::manage_role(Origin::signed(signer), request).unwrap_err();
            assert_eq!(error_msg(error), raw_error_msg);
        }
    });
}

#[test]
fn advanced_checks() {
    new_test_runtime().execute_with(|| {
        init_chain();
        let signer = 1;
        let guild_name = [0u8; 32];
        new_guild(signer, guild_name);

        let register_payload = RequestData::Register(vec![]);
        let reqcheck_payload = RequestData::ReqCheck {
            account: signer,
            guild: guild_name,
            role: ROLE_1,
        };

        // register oracle operator
        <Oracle>::register_operator(Origin::signed(signer)).unwrap();

        // register user
        <Guild>::register(Origin::signed(signer), register_payload.clone()).unwrap();
        let mut request_id = 0;
        let request = <Oracle>::request(request_id).unwrap();
        assert_eq!(request.requester, signer);
        assert_eq!(request.operator, signer);
        let request_data = Request::<AccountId>::decode(&mut request.data.as_slice()).unwrap();
        assert_eq!(request_data.data, register_payload);

        <Oracle>::callback(Origin::signed(signer), request_id, vec![u8::from(true)]).unwrap();

        // someone else tries to assign a role to a registered user
        let error =
            <Guild>::manage_role(Origin::signed(signer + 1), reqcheck_payload.clone()).unwrap_err();
        assert_eq!(error_msg(error), "BadOrigin");

        // assign role to registered user
        <Guild>::manage_role(Origin::signed(signer), reqcheck_payload.clone()).unwrap();
        request_id += 1;
        let request = <Oracle>::request(request_id).unwrap();
        assert_eq!(request.requester, signer);
        assert_eq!(request.operator, signer);
        let request_data = Request::<AccountId>::decode(&mut request.data.as_slice()).unwrap();
        assert_eq!(request_data.data, reqcheck_payload.clone());
        <Oracle>::callback(Origin::signed(signer), request_id, vec![u8::from(true)]).unwrap();

        // strip role request successfully submitted by any signer
        <Guild>::manage_role(Origin::signed(signer + 1), reqcheck_payload.clone()).unwrap();
        request_id += 1;
        let request = <Oracle>::request(request_id).unwrap();
        assert_eq!(request.requester, signer + 1);
        assert_eq!(request_data.data, reqcheck_payload.clone());

        // strip role request immediately executed if the signer is the same
        // without any oracle involvement
        <Guild>::manage_role(Origin::signed(signer), reqcheck_payload).unwrap();
        request_id += 1;
        assert!(<Oracle>::request(request_id).is_none());
    });
}

#[test]
fn storage_checks() {
    new_test_runtime().execute_with(|| {
        init_chain();
        let signer = 1;
        let guild_name = [0u8; 32];
        new_guild(signer, guild_name);

        <Oracle>::register_operator(Origin::signed(signer)).unwrap();
    });
}

/*
#[test]
fn joining_a_guild() {
    new_test_runtime().execute_with(|| {
        init_chain();
        let guild_name = [0u8; 32];
        let role_1_name = [1u8; 32];
        let role_2_name = [2u8; 32];
        let signer = 1;
        let mut request_id = 0;

        <Guild>::create_guild(
            Origin::signed(signer),
            guild_name,
            vec![],
            vec![
                (role_1_name, (vec![], vec![])),
                (role_2_name, (vec![], vec![])),
            ],
        )
        .unwrap();

        // register first
        <Guild>::register(Origin::signed(signer), RequestData::Register(vec![])).unwrap();

        // registration = ok
        <Oracle>::callback(Origin::signed(signer), request_id, vec![u8::from(true)]).unwrap();
        assert!(<Guild>::user_data(signer).is_some());
        request_id += 1;

        // join first role
        <Guild>::assign_role(
            Origin::signed(signer),
            RequestData::ReqCheck {
                guild: guild_name,
                role: role_1_name,
            },
        )
        .unwrap();

        // access = true
        <Oracle>::callback(Origin::signed(signer), request_id, vec![u8::from(true)]).unwrap();
        request_id += 1;

        assert_eq!(
            last_event(),
            Event::Guild(pallet_guild::Event::RoleAssigned(
                signer,
                guild_name,
                role_1_name
            ))
        );

        let guild_id = <Guild>::guild_id(guild_name).unwrap();
        let role_1_id = <Guild>::role_id(guild_id, role_1_name).unwrap();
        let role_2_id = <Guild>::role_id(guild_id, role_2_name).unwrap();
        assert!(<Guild>::member(role_1_id, signer).is_some());
        assert!(<Guild>::member(role_2_id, signer).is_none());
        assert_eq!(<Guild>::user_data(signer), Some(vec![]));

        // try join second role
        <Guild>::assign_role(
            Origin::signed(signer),
            RequestData::ReqCheck {
                guild: guild_name,
                role: role_2_name,
            },
        )
        .unwrap();

        // access = false
        let error = <Oracle>::callback(Origin::signed(signer), request_id, vec![u8::from(false)])
            .unwrap_err();
        assert_eq!(error_msg(error), "AccessDenied");
        request_id += 1;

        assert!(<Guild>::member(role_1_id, signer).is_some());
        assert!(<Guild>::member(role_2_id, signer).is_none());

        // try join second role again
        <Guild>::assign_role(
            Origin::signed(signer),
            RequestData::ReqCheck {
                guild: guild_name,
                role: role_2_name,
            },
        )
        .unwrap();

        // access = true
        <Oracle>::callback(Origin::signed(signer), request_id, vec![u8::from(true)]).unwrap();
        assert!(<Guild>::member(role_1_id, signer).is_some());
        assert!(<Guild>::member(role_2_id, signer).is_some());

        assert_eq!(
            last_event(),
            Event::Guild(pallet_guild::Event::RoleAssigned(
                signer,
                guild_name,
                role_2_name
            ))
        );

        assert_eq!(<Guild>::user_data(signer), Some(vec![]));
    });
}

#[test]
fn joining_the_same_role_in_a_guild_twice_fails() {
    new_test_runtime().execute_with(|| {
        init_chain();
        let guild_name = [0u8; 32];
        let role_name = [1u8; 32];
        let signer = 1;
        let mut request_id = 0;

        <Oracle>::register_operator(Origin::signed(signer)).unwrap();
        <Guild>::create_guild(
            Origin::signed(signer),
            guild_name,
            vec![],
            vec![(role_name, (vec![], vec![]))],
        )
        .unwrap();
        // register first
        <Guild>::register(Origin::signed(signer), RequestData::Register(vec![])).unwrap();
        <Oracle>::callback(Origin::signed(signer), request_id, vec![u8::from(true)]).unwrap();
        request_id += 1;

        // join first time
        <Guild>::assign_role(
            Origin::signed(signer),
            RequestData::ReqCheck {
                guild: guild_name,
                role: role_name,
            },
        )
        .unwrap();

        <Oracle>::callback(Origin::signed(signer), request_id, vec![u8::from(true)]).unwrap();

        let guild_id = <Guild>::guild_id(guild_name).unwrap();
        let role_id = <Guild>::role_id(guild_id, role_name).unwrap();
        assert!(<Guild>::member(role_id, signer).is_some());

        // try to join again
        let error = <Guild>::assign_role(
            Origin::signed(signer),
            RequestData::ReqCheck {
                guild: guild_name,
                role: role_name,
            },
        )
        .unwrap_err();

        assert_eq!(error_msg(error), "RoleAlreadyAssigned");
        assert!(<Guild>::member(role_id, signer).is_some());
    });
}

#[test]
fn joining_multiple_guilds() {
    new_test_runtime().execute_with(|| {
        init_chain();
        let guild_1_name = [1u8; 32];
        let guild_2_name = [2u8; 32];
        let role_1_name = [1u8; 32];
        let role_2_name = [2u8; 32];
        let role_3_name = [3u8; 32];
        let role_4_name = [4u8; 32];
        let signer_1 = 1;
        let signer_2 = 2;

        let user_1_auth = vec![IdentityWithAuth::EvmChain([0; 20], [9; 65])];
        let user_2_auth = vec![IdentityWithAuth::Discord(999, ())];

        let user_1_id = vec![Identity::EvmChain([0; 20])];
        let user_2_id = vec![Identity::Discord(999)];

        <Oracle>::register_operator(Origin::signed(signer_1)).unwrap();

        // create first guild
        <Guild>::create_guild(
            Origin::signed(signer_1),
            guild_1_name,
            vec![],
            vec![
                (role_1_name, (vec![], vec![])),
                (role_2_name, (vec![], vec![])),
            ],
        )
        .unwrap();

        // create second guild
        <Guild>::create_guild(
            Origin::signed(signer_2),
            guild_2_name,
            vec![],
            vec![
                (role_3_name, (vec![], vec![])),
                (role_4_name, (vec![], vec![])),
            ],
        )
        .unwrap();

        // register both users
        <Guild>::register(Origin::signed(signer_1), RequestData::Register(user_1_auth)).unwrap();
        <Guild>::register(Origin::signed(signer_2), RequestData::Register(user_2_auth)).unwrap();

        // registrations
        <Oracle>::callback(Origin::signed(signer_1), 0, vec![u8::from(true)]).unwrap();
        <Oracle>::callback(Origin::signed(signer_1), 1, vec![u8::from(true)]).unwrap();

        // signer 1 wants to join both guilds
        <Guild>::assign_role(
            Origin::signed(signer_1),
            RequestData::ReqCheck {
                guild: guild_1_name,
                role: role_2_name,
            },
        )
        .unwrap();
        <Guild>::assign_role(
            Origin::signed(signer_1),
            RequestData::ReqCheck {
                guild: guild_2_name,
                role: role_3_name,
            },
        )
        .unwrap();

        // signer 2 wants to join both guilds
        <Guild>::assign_role(
            Origin::signed(signer_2),
            RequestData::ReqCheck {
                guild: guild_2_name,
                role: role_4_name,
            },
        )
        .unwrap();
        <Guild>::assign_role(
            Origin::signed(signer_2),
            RequestData::ReqCheck {
                guild: guild_1_name,
                role: role_1_name,
            },
        )
        .unwrap();

        // join requests
        <Oracle>::callback(Origin::signed(signer_1), 2, vec![u8::from(true)]).unwrap();
        <Oracle>::callback(Origin::signed(signer_1), 3, vec![u8::from(true)]).unwrap();
        let error =
            <Oracle>::callback(Origin::signed(signer_1), 4, vec![u8::from(false)]).unwrap_err();
        assert_eq!(error_msg(error), "AccessDenied");
        <Oracle>::callback(Origin::signed(signer_1), 5, vec![u8::from(true)]).unwrap();

        let guild_1_id = <Guild>::guild_id(guild_1_name).unwrap();
        let guild_2_id = <Guild>::guild_id(guild_2_name).unwrap();
        let role_1_id = <Guild>::role_id(guild_1_id, role_1_name).unwrap();
        let role_2_id = <Guild>::role_id(guild_1_id, role_2_name).unwrap();
        let role_3_id = <Guild>::role_id(guild_2_id, role_3_name).unwrap();
        let role_4_id = <Guild>::role_id(guild_2_id, role_4_name).unwrap();

        // 0th request passes
        assert!(<Guild>::member(role_2_id, signer_1).is_some());
        // 1st request passes
        assert!(<Guild>::member(role_3_id, signer_1).is_some());
        // 2nd request fails
        assert!(<Guild>::member(role_4_id, signer_2).is_none());
        // 3rd request passes
        assert!(<Guild>::member(role_1_id, signer_2).is_some());

        assert_eq!(<Guild>::user_data(signer_1), Some(user_1_id));
        assert_eq!(<Guild>::user_data(signer_2), Some(user_2_id));
    });
}
*/
