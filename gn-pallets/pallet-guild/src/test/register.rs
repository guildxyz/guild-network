use super::*;

#[test]
fn successful_registration() {
    new_test_runtime().execute_with(|| {
        init_chain();
        let operator = 0;
        let user_1 = 1;
        let user_2 = 2;

        <Oracle>::register_operator(Origin::signed(operator)).unwrap();

        // wrong request data variant
        let error = <Guild>::register(
            Origin::signed(operator),
            RequestData::ReqCheck {
                account: 1,
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

        assert_eq!(
            last_event(),
            Event::Guild(pallet_guild::Event::UserRegistered(user_1))
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
        assert_eq!(
            last_event(),
            Event::Guild(pallet_guild::Event::UserRegistered(user_2))
        );
    });
}

#[test]
fn invalid_multiple_type() {
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
