use super::*;

#[test]
fn basic_checks() {
    new_test_ext().execute_with(|| {
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
            let error = <Guild>::manage_role(RuntimeOrigin::signed(signer), request).unwrap_err();
            assert_eq!(error_msg(error), raw_error_msg);
        }
    });
}

#[test]
fn advanced_checks() {
    new_test_ext().execute_with(|| {
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

        let mut request_id = 0;

        // register a single operator
        <Oracle>::register_operator(RuntimeOrigin::signed(signer_1)).unwrap();
        // register both users to guild
        <Guild>::register(RuntimeOrigin::signed(signer_1), RequestData::Register(vec![])).unwrap();
        <Guild>::register(RuntimeOrigin::signed(signer_2), RequestData::Register(vec![])).unwrap();

        // both register requests are accepted
        <Oracle>::callback(RuntimeOrigin::signed(signer_1), request_id, vec![u8::from(true)]).unwrap();
        request_id += 1;
        <Oracle>::callback(RuntimeOrigin::signed(signer_1), request_id, vec![u8::from(true)]).unwrap();
        request_id += 1;
        assert!(<Guild>::user_data(signer_1).is_some());
        assert!(<Guild>::user_data(signer_2).is_some());

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
