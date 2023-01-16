use crate::{self as pallet_guild};

use codec::{Decode, Encode};
use frame_support::traits::{OnFinalize, OnInitialize};
use gn_common::{
    identities::{Identity, IdentityWithAuth},
    Request, RequestData,
};
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

const STARTING_BLOCK_NUM: u64 = 2;

fn init_chain() {
    for i in 0..STARTING_BLOCK_NUM {
        System::set_block_number(i);
        <RandomnessCollectiveFlip as OnInitialize<u64>>::on_initialize(i);
        <RandomnessCollectiveFlip as OnFinalize<u64>>::on_finalize(i);
    }
}

fn error_msg<'a>(error: DispatchError) -> &'a str {
    match error {
        DispatchError::Module(module_error) => module_error.message.unwrap(),
        _ => panic!("unexpected error"),
    }
}

fn dummy_answer(
    result: Vec<u8>,
    requester: AccountId,
    request_data: RequestData,
) -> pallet_oracle::OracleAnswer {
    let data = gn_common::Request::<AccountId> {
        requester,
        data: request_data,
    }
    .encode();
    pallet_oracle::OracleAnswer { data, result }
}

#[test]
fn create_guild() {
    new_test_runtime().execute_with(|| {
        init_chain();
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
        assert_eq!(guild.data.owner, signer);
        assert_eq!(guild.data.metadata, metadata);

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
        init_chain();
        let error = <Guild>::callback(Origin::signed(1), vec![]).unwrap_err();
        assert_eq!(error, DispatchError::BadOrigin,);

        let error = <Guild>::callback(Origin::root(), vec![1]).unwrap_err();
        assert_eq!(error_msg(error), "CodecError");

        let no_access = dummy_answer(vec![u8::from(false)], 1, RequestData::Register(vec![]));
        let error = <Guild>::callback(Origin::root(), no_access.encode()).unwrap_err();
        assert_eq!(error_msg(error), "AccessDenied");

        let access = dummy_answer(
            vec![u8::from(true)],
            2,
            RequestData::Join {
                guild: [0; 32],
                role: [1; 32],
            },
        );
        let error = <Guild>::callback(Origin::root(), access.encode()).unwrap_err();
        assert_eq!(error_msg(error), "GuildDoesNotExist");
    });
}

#[test]
fn register_user() {
    new_test_runtime().execute_with(|| {
        init_chain();
        let operator = 0;
        let user_1 = 1;
        let user_2 = 2;

        <Oracle>::register_operator(Origin::signed(operator)).unwrap();

        // wrong request data variant
        let error = <Guild>::register(
            Origin::signed(operator),
            RequestData::Join {
                guild: [0; 32],
                role: [1; 32],
            },
        )
        .unwrap_err();
        assert_eq!(error_msg(error), "InvalidRequestData");

        // register without identities
        let identities_with_auth = vec![];
        let request_data = RequestData::Register(identities_with_auth);
        <Guild>::register(Origin::signed(user_1), request_data.clone()).unwrap();
        let answer = dummy_answer(vec![u8::from(true)], user_1, request_data);
        <Guild>::callback(Origin::root(), answer.encode()).unwrap();
        assert_eq!(<Guild>::user_data(&user_1), Some(vec![]));

        // register identities for already registered user
        let identities_with_auth = vec![
            IdentityWithAuth::EvmChain([0; 20], [1; 65]),
            IdentityWithAuth::Discord(123, ()),
        ];
        let request_data = RequestData::Register(identities_with_auth);
        <Guild>::register(Origin::signed(user_1), request_data.clone()).unwrap();
        let answer = dummy_answer(vec![u8::from(true)], user_1, request_data);
        <Guild>::callback(Origin::root(), answer.encode()).unwrap();
        assert_eq!(
            <Guild>::user_data(&user_1),
            Some(vec![Identity::EvmChain([0; 20]), Identity::Discord(123)])
        );

        // re-register identities but only new ones are pushed
        // NOTE: this behavior should be purposefully broken
        let identities_with_auth = vec![
            IdentityWithAuth::EvmChain([0; 20], [1; 65]),
            IdentityWithAuth::Telegram(99, ()),
        ];
        let request_data = RequestData::Register(identities_with_auth);
        let error = <Guild>::register(Origin::signed(user_1), request_data.clone()).unwrap_err();
        assert_eq!(error_msg(error), "IdentityTypeAlreadyExists");

        let answer = dummy_answer(vec![u8::from(true)], user_1, request_data);
        <Guild>::callback(Origin::root(), answer.encode()).unwrap();
        assert_eq!(
            <Guild>::user_data(&user_1),
            Some(vec![
                Identity::EvmChain([0; 20]),
                Identity::Discord(123),
                Identity::Telegram(99)
            ])
        );

        // register all identities at once
        let identities_with_auth = vec![
            IdentityWithAuth::EvmChain([11; 20], [92; 65]),
            IdentityWithAuth::Discord(12, ()),
            IdentityWithAuth::Telegram(33, ()),
        ];
        let request_data = RequestData::Register(identities_with_auth);
        <Guild>::register(Origin::signed(user_2), request_data.clone()).unwrap();
        let answer = dummy_answer(vec![u8::from(true)], user_2, request_data);
        <Guild>::callback(Origin::root(), answer.encode()).unwrap();
        assert_eq!(
            <Guild>::user_data(&user_2),
            Some(vec![
                Identity::EvmChain([11; 20]),
                Identity::Discord(12),
                Identity::Telegram(33)
            ])
        );
    });
}

#[test]
fn invalid_multiple_type_register() {
    new_test_runtime().execute_with(|| {
        init_chain();
        let operator = 0;
        let user = 2;

        <Oracle>::register_operator(Origin::signed(operator)).unwrap();

        let identities_with_auth = vec![
            IdentityWithAuth::EvmChain([11; 20], [92; 65]),
            IdentityWithAuth::EvmChain([11; 20], [92; 65]),
            IdentityWithAuth::Discord(12, ()),
            IdentityWithAuth::Telegram(33, ()),
        ];
        let request_data = RequestData::Register(identities_with_auth);
        let error = <Guild>::register(Origin::signed(user), request_data).unwrap_err();
        assert_eq!(error_msg(error), "InvalidRequestData");
    });
}

#[test]
fn invalid_join_guild_request() {
    new_test_runtime().execute_with(|| {
        init_chain();
        let signer = 1;
        let guild_name = [0u8; 32];
        let role_name = [1u8; 32];
        let request_data = RequestData::Join {
            guild: [111; 32],
            role: [255; 32],
        };

        // try join with invalid data variant
        let error =
            <Guild>::join_guild(Origin::signed(signer), RequestData::Register(vec![])).unwrap_err();
        assert_eq!(error_msg(error), "InvalidRequestData");
        // try join non-existend guild
        let error = <Guild>::join_guild(Origin::signed(signer), request_data).unwrap_err();
        assert_eq!(error_msg(error), "GuildDoesNotExist");
        assert_eq!(<Guild>::next_request_id(), 0);

        // create the actual guild
        <Guild>::create_guild(
            Origin::signed(signer),
            guild_name,
            vec![],
            vec![(role_name, vec![])],
        )
        .unwrap();

        // try join existing guild with invalid role
        let request_data = RequestData::Join {
            guild: guild_name,
            role: [255; 32],
        };
        let error = <Guild>::join_guild(Origin::signed(signer), request_data).unwrap_err();
        assert_eq!(error_msg(error), "RoleDoesNotExist");
        //  try join without registering first
        let request_data = RequestData::Join {
            guild: guild_name,
            role: role_name,
        };
        let error = <Guild>::join_guild(Origin::signed(signer), request_data).unwrap_err();
        assert_eq!(error_msg(error), "UserNotRegistered");
    });
}

#[test]
fn valid_join_guild_request() {
    new_test_runtime().execute_with(|| {
        init_chain();
        let guild_name = [0u8; 32];
        let role_name = [1u8; 32];
        let signer = 1;
        let register = RequestData::Register(vec![]);
        let join = RequestData::Join {
            guild: guild_name,
            role: role_name,
        };

        <Oracle>::register_operator(Origin::signed(signer)).unwrap();
        <Guild>::create_guild(
            Origin::signed(signer),
            guild_name,
            vec![],
            vec![(role_name, vec![])],
        )
        .unwrap();

        <Guild>::register(Origin::signed(signer), register.clone()).unwrap();

        let mut request_id = 0;
        let request = <Oracle>::request(request_id).unwrap();
        assert_eq!(request.requester, signer);
        assert_eq!(request.operator, signer);
        let request_data = Request::<AccountId>::decode(&mut request.data.as_slice()).unwrap();
        assert_eq!(request_data.data, register);

        <Oracle>::callback(Origin::signed(signer), request_id, vec![u8::from(true)]).unwrap();

        <Guild>::join_guild(Origin::signed(signer), join.clone()).unwrap();
        request_id += 1;

        let request = <Oracle>::request(request_id).unwrap();
        assert_eq!(request.requester, signer);
        assert_eq!(request.operator, signer);
        let request_data = Request::<AccountId>::decode(&mut request.data.as_slice()).unwrap();
        assert_eq!(request_data.data, join);
    });
}

#[test]
fn joining_a_guild() {
    new_test_runtime().execute_with(|| {
        init_chain();
        let guild_name = [0u8; 32];
        let role_1_name = [1u8; 32];
        let role_2_name = [2u8; 32];
        let signer = 1;
        let mut request_id = 0;

        <Oracle>::register_operator(Origin::signed(signer)).unwrap();
        <Guild>::create_guild(
            Origin::signed(signer),
            guild_name,
            vec![],
            vec![(role_1_name, vec![]), (role_2_name, vec![])],
        )
        .unwrap();

        // register first
        <Guild>::register(Origin::signed(signer), RequestData::Register(vec![])).unwrap();

        // registration = ok
        <Oracle>::callback(Origin::signed(signer), request_id, vec![u8::from(true)]).unwrap();
        assert!(<Guild>::user_data(signer).is_some());
        request_id += 1;

        // join first role
        <Guild>::join_guild(
            Origin::signed(signer),
            RequestData::Join {
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
            Event::Guild(pallet_guild::Event::GuildJoined(
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
        <Guild>::join_guild(
            Origin::signed(signer),
            RequestData::Join {
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
        <Guild>::join_guild(
            Origin::signed(signer),
            RequestData::Join {
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
            Event::Guild(pallet_guild::Event::GuildJoined(
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
            vec![(role_name, vec![])],
        )
        .unwrap();
        // register first
        <Guild>::register(Origin::signed(signer), RequestData::Register(vec![])).unwrap();
        <Oracle>::callback(Origin::signed(signer), request_id, vec![u8::from(true)]).unwrap();
        request_id += 1;

        // join first time
        <Guild>::join_guild(
            Origin::signed(signer),
            RequestData::Join {
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
        let error = <Guild>::join_guild(
            Origin::signed(signer),
            RequestData::Join {
                guild: guild_name,
                role: role_name,
            },
        )
        .unwrap_err();

        assert_eq!(error_msg(error), "UserAlreadyJoined");
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

        // register both users
        <Guild>::register(Origin::signed(signer_1), RequestData::Register(user_1_auth)).unwrap();
        <Guild>::register(Origin::signed(signer_2), RequestData::Register(user_2_auth)).unwrap();

        // registrations
        <Oracle>::callback(Origin::signed(signer_1), 0, vec![u8::from(true)]).unwrap();
        <Oracle>::callback(Origin::signed(signer_1), 1, vec![u8::from(true)]).unwrap();

        // signer 1 wants to join both guilds
        <Guild>::join_guild(
            Origin::signed(signer_1),
            RequestData::Join {
                guild: guild_1_name,
                role: role_2_name,
            },
        )
        .unwrap();
        <Guild>::join_guild(
            Origin::signed(signer_1),
            RequestData::Join {
                guild: guild_2_name,
                role: role_3_name,
            },
        )
        .unwrap();

        // signer 2 wants to join both guilds
        <Guild>::join_guild(
            Origin::signed(signer_2),
            RequestData::Join {
                guild: guild_2_name,
                role: role_4_name,
            },
        )
        .unwrap();
        <Guild>::join_guild(
            Origin::signed(signer_2),
            RequestData::Join {
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
