use crate as pallet_guild;
use test_runtime::test_runtime;

use sp_runtime::DispatchError;

test_runtime!(Guild, pallet_guild);

pub fn last_event() -> Event {
    System::events()
        .into_iter()
        .map(|r| r.event)
        .filter_map(|e| {
            if let Event::Guild(inner) = e {
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
        let guild_id = [0u8; 32];
        let metadata = vec![1, 2, 3, 4, 5];

        let role_1_id = [1u8; 32];
        let role_2_id = [2u8; 32];
        let role_3_id = [3u8; 32];

        let role_1_data = vec![6, 7, 8, 9, 0];
        let role_2_data = vec![2, 4, 6, 8, 0];
        let role_3_data = vec![1, 3, 5, 7, 9];

        let roles = vec![
            (role_1_id, role_1_data.clone()),
            (role_2_id, role_2_data.clone()),
            (role_3_id, role_3_data.clone()),
        ];

        assert!(
            <Guild>::create_guild(Origin::signed(signer), guild_id, metadata.clone(), roles)
                .is_ok()
        );

        assert_eq!(
            last_event(),
            Event::Guild(pallet_guild::Event::GuildCreated(signer, guild_id))
        );

        let guild = <Guild>::guild(guild_id).unwrap();
        assert_eq!(guild.owner, signer);
        assert_eq!(guild.metadata, metadata);

        let expected_role_1_data = <Guild>::role(guild_id, role_1_id).unwrap();
        let expected_role_2_data = <Guild>::role(guild_id, role_2_id).unwrap();
        let expected_role_3_data = <Guild>::role(guild_id, role_3_id).unwrap();

        assert_eq!(expected_role_1_data, role_1_data);
        assert_eq!(expected_role_2_data, role_2_data);
        assert_eq!(expected_role_3_data, role_3_data);
    });
}

#[test]
fn callback_can_only_be_called_by_root() {
    new_test_runtime().execute_with(|| {
        System::set_block_number(1);
        let error = <Guild>::callback(Origin::signed(1), vec![]).err().unwrap();
        assert_eq!(error, DispatchError::BadOrigin,);

        let error = <Guild>::callback(Origin::root(), vec![]).err().unwrap();
        assert_eq!(error_msg(error), "InvalidResultLength");

        let error = <Guild>::callback(
            Origin::root(),
            vec![255, 255, 255, 255, 255, 255, 255, 255, 1],
        )
        .err()
        .unwrap();
        assert_eq!(error_msg(error), "JoinRequestDoesNotExist");

        assert_eq!(
            last_event(),
            Event::Guild(pallet_guild::Event::OracleResult(u64::MAX, true))
        );

        let error = <Guild>::callback(Origin::root(), vec![0, 0, 0, 0, 0, 0, 0, 0, 0])
            .err()
            .unwrap();
        assert_eq!(error_msg(error), "JoinRequestDoesNotExist");
        assert_eq!(
            last_event(),
            Event::Guild(pallet_guild::Event::OracleResult(0, false))
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
        assert_eq!(join_request.guild_id, guild_id);
        assert_eq!(join_request.role_id, role_id);

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
        assert_eq!(join_request.guild_id, guild_id);
        assert_eq!(join_request.role_id, role_id);
    });
}

#[test]
fn joining_a_guild() {}

#[test]
fn joining_multiple_guilds() {}
