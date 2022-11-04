use crate::{self as pallet_guild, GuildIdMap, RoleIdMap};

use frame_support::traits::OnFinalize;
use sp_runtime::DispatchError;

use test_runtime::test_runtime;
test_runtime!(Guild, pallet_guild);

pub fn last_event() -> Event {
    System::events()
        .into_iter()
        .filter_map(|e| {
            if let Event::Guild(inner) = e.event {
                Some(Event::Guild(inner))
            } else {
                None
            }
        })
        .last()
        .unwrap()
}

fn error_msg<'a>(error: DispatchError) -> &'a str {
    match error {
        DispatchError::Module(module_error) => module_error.message.unwrap(),
        _ => panic!("unexpected error"),
    }
}

// TODO add more tests once guild functionalities are final
#[test]
fn create_guild() {
    new_test_runtime().execute_with(|| {
        System::set_block_number(1);
        let signer = 4;
        let guild_name = [0u8; 32];
        let metadata = vec![1, 2, 3, 4, 5];

        let role_1_name = [1u8; 32];
        let role_2_name = [2u8; 32];
        let role_3_name = [3u8; 32];

        let role_1_data = vec![6, 7, 8, 9, 0];
        let role_2_data = vec![2, 4, 6, 8, 0];
        let role_3_data = vec![1, 3, 5, 7, 9];

        let roles = vec![
            (role_1_name, role_1_data.clone()),
            (role_2_name, role_2_data.clone()),
            (role_3_name, role_3_data.clone()),
        ];

        assert!(
            <Guild>::create_guild(Origin::signed(signer), guild_name, metadata.clone(), roles)
                .is_ok()
        );

        assert_eq!(
            last_event(),
            Event::Guild(pallet_guild::Event::GuildCreated(signer, guild_name))
        );

        let guild_id = <Guild>::guild_id(guild_name).unwrap();
        let guild = <Guild>::guild(guild_id).unwrap();
        assert_eq!(guild.owner, signer);
        assert_eq!(guild.metadata, metadata);

        let role_1_id = <Guild>::role_id(guild_id, role_1_name).unwrap();
        let role_2_id = <Guild>::role_id(guild_id, role_2_name).unwrap();
        let role_3_id = <Guild>::role_id(guild_id, role_3_name).unwrap();
        let expected_role_1_data = <Guild>::role(role_1_id).unwrap();
        let expected_role_2_data = <Guild>::role(role_2_id).unwrap();
        let expected_role_3_data = <Guild>::role(role_3_id).unwrap();

        assert_eq!(expected_role_1_data, role_1_data);
        assert_eq!(expected_role_2_data, role_2_data);
        assert_eq!(expected_role_3_data, role_3_data);
    });
}

#[test]
fn callback_can_only_be_called_by_root() {
    new_test_runtime().execute_with(|| {
        System::set_block_number(1);
        let error = <Guild>::callback(Origin::signed(1), false, vec![])
            .err()
            .unwrap();
        assert_eq!(error, DispatchError::BadOrigin,);

        let error = <Guild>::callback(Origin::root(), false, vec![])
            .err()
            .unwrap();
        assert_eq!(error_msg(error), "InvalidResultLength");

        let error = <Guild>::callback(
            Origin::root(),
            false,
            vec![255, 255, 255, 255, 255, 255, 255, 255, 1],
        )
        .err()
        .unwrap();
        assert_eq!(error_msg(error), "JoinRequestDoesNotExist");

        assert_eq!(
            last_event(),
            Event::Guild(pallet_guild::Event::OracleResult(u64::MAX, true))
        );

        let error = <Guild>::callback(Origin::root(), false, vec![0, 0, 0, 0, 0, 0, 0, 0, 0])
            .err()
            .unwrap();
        assert_eq!(error_msg(error), "JoinRequestDoesNotExist");
        assert_eq!(
            last_event(),
            Event::Guild(pallet_guild::Event::OracleResult(0, false))
        );

        // expired requests don't even check whether the join request exits, it simply
        // attempts to remove it
        <Guild>::callback(Origin::root(), true, vec![0, 0, 0, 0, 0, 0, 0, 0, 0]).unwrap();
        assert_eq!(
            last_event(),
            Event::Guild(pallet_guild::Event::JoinRequestExpired(0))
        );
    });
}

#[test]
fn invalid_join_guild_request() {
    new_test_runtime().execute_with(|| {
        System::set_block_number(1);
        let guild_id = [0u8; 32];
        let role_id = [1u8; 32];

        // try join non-existent guild
        let error = <Guild>::join_guild(Origin::signed(1), guild_id, role_id, vec![], vec![])
            .err()
            .unwrap();
        assert_eq!(error_msg(error), "InvalidGuildRole");
        assert_eq!(<Guild>::next_request_id(), 0);

        // try join existing guild with invalid role
        <Guild>::create_guild(Origin::signed(1), guild_id, vec![], vec![]).unwrap();
        let error = <Guild>::join_guild(Origin::signed(1), guild_id, role_id, vec![], vec![])
            .err()
            .unwrap();
        assert_eq!(error_msg(error), "InvalidGuildRole");
    });
}

#[test]
fn valid_join_guild_request() {
    new_test_runtime().execute_with(|| {
        System::set_block_number(1);
        let guild_id = [0u8; 32];
        let role_id = [1u8; 32];

        <Chainlink>::register_operator(Origin::signed(1)).unwrap();
        <Guild>::create_guild(Origin::signed(1), guild_id, vec![], vec![(role_id, vec![])])
            .unwrap();
        <Guild>::join_guild(
            Origin::signed(1),
            guild_id,
            role_id,
            vec![1, 2, 3],
            vec![4, 5, 6],
        )
        .unwrap();
        assert_eq!(<Guild>::next_request_id(), 1);
        let join_request = <Guild>::join_request(0).unwrap();
        assert_eq!(join_request.requester, 1);
        assert_eq!(join_request.requester_identities, vec![1, 2, 3]);
        assert_eq!(join_request.guild_name, guild_id);
        assert_eq!(join_request.role_name, role_id);

        <Guild>::join_guild(
            Origin::signed(2),
            guild_id,
            role_id,
            vec![1, 2, 3],
            vec![4, 5, 6],
        )
        .unwrap();
        assert_eq!(<Guild>::next_request_id(), 2);
        let join_request = <Guild>::join_request(1).unwrap();
        assert_eq!(join_request.requester, 2);
        assert_eq!(join_request.requester_identities, vec![1, 2, 3]);
        assert_eq!(join_request.guild_name, guild_id);
        assert_eq!(join_request.role_name, role_id);
    });
}

#[test]
fn joining_a_guild() {
    new_test_runtime().execute_with(|| {
        System::set_block_number(1);
        let guild_name = [0u8; 32];
        let role_1_name = [1u8; 32];
        let role_2_name = [2u8; 32];

        let signer = 1;
        let user_data = vec![1, 2, 3];

        <Chainlink>::register_operator(Origin::signed(signer)).unwrap();
        <Guild>::create_guild(
            Origin::signed(signer),
            guild_name,
            vec![],
            vec![(role_1_name, vec![]), (role_2_name, vec![])],
        )
        .unwrap();
        // join first role
        <Guild>::join_guild(
            Origin::signed(signer),
            guild_name,
            role_1_name,
            user_data.clone(),
            vec![],
        )
        .unwrap();

        assert!(<Guild>::user_data(signer).is_none());

        // access = true
        <Chainlink>::callback(Origin::signed(signer), 0, vec![1]).unwrap();

        assert_eq!(
            last_event(),
            Event::Guild(pallet_guild::Event::GuildJoined(
                signer,
                guild_name,
                role_1_name
            ))
        );

        let guild_id = <Guild>::guild_id(guild_name).unwrap();
        let role_1_id = <Guild>::role_id(guild_id, role_1_name).unwrap();
        let role_2_id = <Guild>::role_id(guild_id, role_2_name).unwrap();
        assert!(<Guild>::join_request(0).is_none());
        assert!(<Guild>::member(role_1_id, signer).is_some());
        assert!(<Guild>::member(role_2_id, signer).is_none());
        assert_eq!(<Guild>::user_data(signer).unwrap(), user_data);

        // try join second role
        <Guild>::join_guild(
            Origin::signed(signer),
            guild_name,
            role_2_name,
            user_data.clone(),
            vec![],
        )
        .unwrap();
        assert!(<Guild>::join_request(0).is_none());
        assert!(<Guild>::join_request(1).is_some());

        // access = false
        <Chainlink>::callback(Origin::signed(signer), 1, vec![0]).unwrap();
        assert!(<Guild>::join_request(1).is_none());
        assert!(<Guild>::member(role_1_id, signer).is_some());
        assert!(<Guild>::member(role_2_id, signer).is_none());

        assert_eq!(
            last_event(),
            Event::Guild(pallet_guild::Event::AccessDenied(
                signer,
                guild_name,
                role_2_name
            ))
        );

        // try join second role again
        <Guild>::join_guild(
            Origin::signed(signer),
            guild_name,
            role_2_name,
            user_data.clone(),
            vec![],
        )
        .unwrap();
        assert!(<Guild>::join_request(1).is_none());
        assert!(<Guild>::join_request(2).is_some());

        // access = true
        <Chainlink>::callback(Origin::signed(signer), 2, vec![1]).unwrap();
        assert!(<Guild>::join_request(1).is_none());
        assert!(<Guild>::member(role_1_id, signer).is_some());
        assert!(<Guild>::member(role_2_id, signer).is_some());

        assert_eq!(
            last_event(),
            Event::Guild(pallet_guild::Event::GuildJoined(
                signer,
                guild_name,
                role_2_name
            ))
        );

        assert_eq!(<Guild>::user_data(signer).unwrap(), user_data);
    });
}

#[test]
fn joining_a_guild_twice() {
    new_test_runtime().execute_with(|| {
        System::set_block_number(1);
        let guild_name = [0u8; 32];
        let role_name = [1u8; 32];
        let user_data = vec![1, 2, 3];
        let signer = 1;

        <Chainlink>::register_operator(Origin::signed(signer)).unwrap();
        <Guild>::create_guild(
            Origin::signed(signer),
            guild_name,
            vec![],
            vec![(role_name, vec![])],
        )
        .unwrap();
        // join first time
        <Guild>::join_guild(
            Origin::signed(signer),
            guild_name,
            role_name,
            user_data,
            vec![],
        )
        .unwrap();
        <Chainlink>::callback(Origin::signed(signer), 0, vec![1]).unwrap();

        let guild_id = <Guild>::guild_id(guild_name).unwrap();
        let role_id = <Guild>::role_id(guild_id, role_name).unwrap();
        assert!(<Guild>::member(role_id, signer).is_some());

        // try to join again
        <Guild>::join_guild(
            Origin::signed(signer),
            guild_name,
            role_name,
            vec![],
            vec![],
        )
        .unwrap();
        <Chainlink>::callback(Origin::signed(signer), 1, vec![1]).unwrap();
        assert!(<Guild>::member(role_id, signer).is_some());
        assert!(<Guild>::join_request(0).is_none());
        assert!(<Guild>::join_request(1).is_none());
        assert!(<Guild>::join_request(2).is_none());
    });
}

#[test]
fn joining_multiple_guilds() {
    new_test_runtime().execute_with(|| {
        System::set_block_number(1);
        let guild_1_name = [1u8; 32];
        let guild_2_name = [2u8; 32];
        let role_1_name = [1u8; 32];
        let role_2_name = [2u8; 32];
        let role_3_name = [3u8; 32];
        let role_4_name = [4u8; 32];
        let signer_1 = 1;
        let signer_2 = 2;

        let user_1_data = vec![1, 2, 3];
        let user_2_data = vec![4, 5, 6];

        <Chainlink>::register_operator(Origin::signed(signer_1)).unwrap();
        // create first guild
        <Guild>::create_guild(
            Origin::signed(signer_1),
            guild_1_name,
            vec![],
            vec![(role_1_name, vec![]), (role_2_name, vec![])],
        )
        .unwrap();
        // create second guild
        <Guild>::create_guild(
            Origin::signed(signer_2),
            guild_2_name,
            vec![],
            vec![(role_3_name, vec![]), (role_4_name, vec![])],
        )
        .unwrap();
        // signer 1 wants to join both guilds
        <Guild>::join_guild(
            Origin::signed(signer_1),
            guild_1_name,
            role_2_name,
            user_1_data.clone(),
            vec![],
        )
        .unwrap();
        <Guild>::join_guild(
            Origin::signed(signer_1),
            guild_2_name,
            role_3_name,
            user_1_data.clone(),
            vec![],
        )
        .unwrap();
        // signer 2 wants to join both guilds
        <Guild>::join_guild(
            Origin::signed(signer_2),
            guild_2_name,
            role_4_name,
            user_2_data.clone(),
            vec![],
        )
        .unwrap();
        <Guild>::join_guild(
            Origin::signed(signer_2),
            guild_1_name,
            role_1_name,
            user_2_data.clone(),
            vec![],
        )
        .unwrap();

        assert_eq!(<Guild>::next_request_id(), 4);

        <Chainlink>::callback(Origin::signed(signer_1), 3, vec![1]).unwrap();
        <Chainlink>::callback(Origin::signed(signer_1), 0, vec![1]).unwrap();
        <Chainlink>::callback(Origin::signed(signer_1), 2, vec![0]).unwrap();
        <Chainlink>::callback(Origin::signed(signer_1), 1, vec![1]).unwrap();

        let guild_1_id = <Guild>::guild_id(guild_1_name).unwrap();
        let guild_2_id = <Guild>::guild_id(guild_2_name).unwrap();
        let role_1_id = <Guild>::role_id(guild_1_id, role_1_name).unwrap();
        let role_2_id = <Guild>::role_id(guild_2_id, role_2_name).unwrap();
        let role_3_id = <Guild>::role_id(guild_2_id, role_3_name).unwrap();
        let role_4_id = <Guild>::role_id(guild_1_id, role_4_name).unwrap();

        // 0th request passes
        assert!(<Guild>::member(role_2_id, signer_1).is_some());
        // 1st request passes
        assert!(<Guild>::member(role_3_id, signer_1).is_some());
        // 2nd request fails
        assert!(<Guild>::member(role_4_id, signer_2).is_none());
        // 3rd request passes
        assert!(<Guild>::member(role_1_id, signer_2).is_some());

        assert_eq!(<Guild>::user_data(signer_1).unwrap(), user_1_data);
        assert_eq!(<Guild>::user_data(signer_2).unwrap(), user_2_data);
    });
}

#[test]
fn kill_request() {
    new_test_runtime().execute_with(|| {
        let guild_name = [0u8; 32];
        let role_name = [1u8; 32];
        let signer = 1;

        <Chainlink>::register_operator(Origin::signed(signer)).unwrap();
        // create first guild
        <Guild>::create_guild(
            Origin::signed(signer),
            guild_name,
            vec![],
            vec![(role_name, vec![])],
        )
        .unwrap();

        <Guild>::join_guild(
            Origin::signed(signer),
            guild_name,
            role_name,
            vec![],
            vec![],
        )
        .unwrap();
        assert!(<Guild>::join_request(0).is_some());

        <Chainlink as OnFinalize<u64>>::on_finalize(
            <TestRuntime as pallet_chainlink::Config>::ValidityPeriod::get() + 1,
        );

        assert!(<Guild>::join_request(0).is_none());

        let guild_id = <Guild>::guild_id(guild_name).unwrap();
        let role_id = <Guild>::role_id(guild_id, role_name).unwrap();
        assert!(<Guild>::member(role_id, signer).is_none());
    });
}
