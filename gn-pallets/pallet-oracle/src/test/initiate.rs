use super::*;

#[test]
fn initiate_requests_valid() {
    new_test_ext().execute_with(|| {
        let pallet_index = 0;
        let fee = minimum_fee();
        let parameters = ("a", "b");
        let data = parameters.encode();
        let request_id = 0;

        assert_ok!(<Oracle>::register_operator(
            RuntimeOrigin::root(),
            ACCOUNT_0
        ));
        assert_eq!(last_event(), OracleEvent::OperatorRegistered(ACCOUNT_0));
        assert_ok!(<Oracle>::activate_operator(RuntimeOrigin::signed(
            ACCOUNT_0
        )));
        assert_eq!(last_event(), OracleEvent::OperatorActivated(ACCOUNT_0));

        assert_ok!(<Oracle>::initiate_request(
            RuntimeOrigin::signed(ACCOUNT_1),
            pallet_index,
            data.clone(),
            fee,
        ));

        assert_eq!(
            last_event(),
            OracleEvent::OracleRequest {
                request_id,
                operator: ACCOUNT_0,
                pallet_index,
                fee,
            }
        );

        let r = <(Vec<u8>, Vec<u8>)>::decode(&mut &data[..]).unwrap();
        assert_eq!(parameters.0, std::str::from_utf8(&r.0).unwrap());
        assert_eq!(parameters.1, std::str::from_utf8(&r.1).unwrap());

        assert_ok!(<Oracle>::callback(
            RuntimeOrigin::root(),
            ACCOUNT_0,
            request_id
        ));

        assert_eq!(
            last_event(),
            OracleEvent::OracleAnswer {
                request_id,
                operator: ACCOUNT_0,
            }
        );
    });
}

#[test]
fn linear_request_delegation() {
    new_test_ext().execute_with(|| {
        let operator = 9;
        let operator_0 = 10;
        let operator_1 = 11;
        let operator_2 = 12;
        let operator_3 = 13;
        let data = vec![];
        let pallet_index = 0;
        let fee = minimum_fee();
        let mut request_id = 0;

        assert_ok!(<Oracle>::register_operator(RuntimeOrigin::root(), operator));
        assert_ok!(<Oracle>::register_operator(
            RuntimeOrigin::root(),
            operator_0
        ));
        assert_ok!(<Oracle>::register_operator(
            RuntimeOrigin::root(),
            operator_1
        ));
        assert_ok!(<Oracle>::register_operator(
            RuntimeOrigin::root(),
            operator_2
        ));
        assert_noop!(
            <Oracle>::register_operator(RuntimeOrigin::root(), operator_3),
            OracleError::MaxOperatorsRegistered
        );
        assert_eq!(
            <Oracle>::num_registered_operators(),
            <TestRuntime as pallet_oracle::Config>::MaxOperators::get()
        );
        assert_ok!(<Oracle>::deregister_operator(
            RuntimeOrigin::root(),
            operator
        ));
        assert_ok!(<Oracle>::register_operator(
            RuntimeOrigin::root(),
            operator_3
        ));

        // activate operators
        assert_ok!(<Oracle>::activate_operator(RuntimeOrigin::signed(
            operator_0
        )));
        assert_ok!(<Oracle>::activate_operator(RuntimeOrigin::signed(
            operator_1
        )));
        assert_ok!(<Oracle>::activate_operator(RuntimeOrigin::signed(
            operator_2
        )));
        assert_ok!(<Oracle>::activate_operator(RuntimeOrigin::signed(
            operator_3
        )));

        assert_ok!(<Oracle>::initiate_request(
            RuntimeOrigin::signed(ACCOUNT_0),
            pallet_index,
            data.clone(),
            fee,
        ));

        assert_eq!(
            last_event(),
            OracleEvent::OracleRequest {
                request_id,
                operator: operator_0,
                pallet_index,
                fee,
            }
        );
        request_id += 1;

        assert_ok!(<Oracle>::initiate_request(
            RuntimeOrigin::signed(ACCOUNT_0),
            pallet_index,
            data.clone(),
            fee,
        ));

        assert_eq!(
            last_event(),
            OracleEvent::OracleRequest {
                request_id,
                operator: operator_1,
                pallet_index,
                fee,
            }
        );
        request_id += 1;

        assert_ok!(<Oracle>::initiate_request(
            RuntimeOrigin::signed(ACCOUNT_0),
            pallet_index,
            data.clone(),
            fee,
        ));

        assert_eq!(
            last_event(),
            OracleEvent::OracleRequest {
                request_id,
                operator: operator_2,
                pallet_index,
                fee,
            }
        );
        request_id += 1;

        assert_ok!(<Oracle>::initiate_request(
            RuntimeOrigin::signed(ACCOUNT_0),
            pallet_index,
            data.clone(),
            fee,
        ));

        assert_eq!(
            last_event(),
            OracleEvent::OracleRequest {
                request_id,
                operator: operator_3,
                pallet_index,
                fee,
            }
        );
        request_id += 1;

        assert_ok!(<Oracle>::initiate_request(
            RuntimeOrigin::signed(ACCOUNT_0),
            pallet_index,
            data.clone(),
            fee,
        ));

        assert_eq!(
            last_event(),
            OracleEvent::OracleRequest {
                request_id,
                operator: operator_0,
                pallet_index,
                fee,
            }
        );

        request_id += 1;

        // operator_1, and operator_2 deactivates
        assert_ok!(<Oracle>::deactivate_operator(RuntimeOrigin::signed(
            operator_1
        )));
        assert_ok!(<Oracle>::deactivate_operator(RuntimeOrigin::signed(
            operator_2
        )));
        assert_ok!(<Oracle>::initiate_request(
            RuntimeOrigin::signed(ACCOUNT_0),
            pallet_index,
            data.clone(),
            fee,
        ));
        assert_eq!(
            last_event(),
            OracleEvent::OracleRequest {
                request_id,
                operator: operator_3,
                pallet_index,
                fee,
            }
        );

        request_id += 1;

        // operator_0 is deregistered by root
        assert_ok!(<Oracle>::deregister_operator(
            RuntimeOrigin::root(),
            operator_0
        ));
        assert_eq!(<Oracle>::active_operators(), vec![operator_3]);
        assert_ok!(<Oracle>::initiate_request(
            RuntimeOrigin::signed(ACCOUNT_0),
            pallet_index,
            data.clone(),
            fee,
        ));
        assert_eq!(
            last_event(),
            OracleEvent::OracleRequest {
                request_id,
                operator: operator_3,
                pallet_index,
                fee,
            }
        );

        request_id += 1;

        assert_ok!(<Oracle>::initiate_request(
            RuntimeOrigin::signed(ACCOUNT_0),
            pallet_index,
            data,
            fee
        ));
        assert_eq!(
            last_event(),
            OracleEvent::OracleRequest {
                request_id,
                operator: operator_3,
                pallet_index,
                fee,
            }
        );
    });
}

#[test]
fn initiate_requests_invalid_insufficient_fee() {
    new_test_ext().execute_with(|| {
        assert_ok!(<Oracle>::register_operator(
            RuntimeOrigin::root(),
            ACCOUNT_0
        ));
        assert_ok!(<Oracle>::activate_operator(RuntimeOrigin::signed(
            ACCOUNT_0
        )));
        assert_noop!(
            <Oracle>::initiate_request(
                RuntimeOrigin::signed(ACCOUNT_1),
                0,
                vec![],
                minimum_fee() - 1,
            ),
            OracleError::InsufficientFee
        );
    });
}

#[test]
fn initiate_requests_invalid_insufficient_balance_for_fee() {
    new_test_ext().execute_with(|| {
        assert_ok!(<Oracle>::register_operator(
            RuntimeOrigin::root(),
            ACCOUNT_0
        ));
        assert_ok!(<Oracle>::activate_operator(RuntimeOrigin::signed(
            ACCOUNT_0
        )));
        assert_noop!(
            <Oracle>::initiate_request(
                RuntimeOrigin::signed(ACCOUNT_1),
                0,
                vec![],
                GENESIS_BALANCE + 1,
            ),
            pallet_balances::Error::<TestRuntime, _>::InsufficientBalance
        );
    });
}

#[test]
fn initiate_requests_invalid_wrong_operator() {
    new_test_ext().execute_with(|| {
        assert_ok!(<Oracle>::register_operator(
            RuntimeOrigin::root(),
            ACCOUNT_0
        ));
        assert_ok!(<Oracle>::activate_operator(RuntimeOrigin::signed(
            ACCOUNT_0
        )));
        assert_ok!(<Oracle>::initiate_request(
            RuntimeOrigin::signed(ACCOUNT_1),
            0,
            vec![],
            minimum_fee()
        ));
        assert_noop!(
            <Oracle>::callback(RuntimeOrigin::root(), 99, 0),
            OracleError::WrongOperator
        );
    });
}
