mod helpers;
use helpers::*;

use crate::mock::*;
use frame_support::traits::OnFinalize;
use pallet_oracle::Event as OracleEvent;
use parity_scale_codec::{Decode, Encode};

#[test]
fn invalid_transactions_fail() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let failing_transactions = vec![
            (
                <Oracle>::register_operator(RuntimeOrigin::none(), 0),
                "BadOrigin",
            ),
            (
                <Oracle>::register_operator(RuntimeOrigin::signed(0), 0),
                "BadOrigin",
            ),
            (
                <Oracle>::deregister_operator(RuntimeOrigin::none(), 0),
                "BadOrigin",
            ),
            (
                <Oracle>::deregister_operator(RuntimeOrigin::signed(1), 0),
                "BadOrigin",
            ),
            (
                <Oracle>::activate_operator(RuntimeOrigin::signed(1)),
                "UnknownOperator",
            ),
            (
                <Oracle>::deactivate_operator(RuntimeOrigin::signed(1)),
                "UnknownOperator",
            ),
            (
                <Oracle>::deregister_operator(RuntimeOrigin::root(), 1),
                "UnknownOperator",
            ),
            (
                <Oracle>::initiate_request(
                    RuntimeOrigin::signed(1),
                    MockCallback::test(),
                    vec![],
                    minimum_fee(),
                ),
                "NoActiveOperators",
            ),
            (
                <Oracle>::callback(RuntimeOrigin::signed(ACCOUNT_0), 0, 10.encode()),
                "UnknownRequest",
            ),
        ];

        for (tx, raw_error_msg) in failing_transactions {
            assert_eq!(error_msg(tx.unwrap_err()), raw_error_msg);
        }
    });
}

#[test]
fn operator_registration_valid() {
    new_test_ext().execute_with(|| {
        // This is required for some reason otherwise the last_event() method fails
        System::set_block_number(1);
        let operator = 1;
        assert_eq!(<Oracle>::num_registered_operators(), 0);
        assert!(<Oracle>::register_operator(RuntimeOrigin::root(), operator).is_ok());
        assert!(<Oracle>::operator(operator).is_some());
        assert_eq!(<Oracle>::num_registered_operators(), 1);
        assert_eq!(last_event(), OracleEvent::OperatorRegistered(operator));
    });
}

#[test]
fn operator_registration_invalid_operator_already_registered() {
    new_test_ext().execute_with(|| {
        let operator_1 = 1;
        let operator_2 = 2;
        assert!(<Oracle>::register_operator(RuntimeOrigin::root(), operator_1).is_ok());
        assert!(<Oracle>::operator(operator_1).is_some());
        assert_eq!(<Oracle>::num_registered_operators(), 1);

        // Operator already registered error
        let error = <Oracle>::register_operator(RuntimeOrigin::root(), operator_1).unwrap_err();
        assert_eq!(error_msg(error), "OperatorAlreadyRegistered");
        assert!(<Oracle>::operator(operator_1).is_some());
        assert_eq!(<Oracle>::num_registered_operators(), 1);

        assert!(<Oracle>::register_operator(RuntimeOrigin::root(), operator_2).is_ok());
        assert!(<Oracle>::operator(operator_1).is_some());
        assert!(<Oracle>::operator(operator_2).is_some());
        assert_eq!(<Oracle>::num_registered_operators(), 2);
    });
}

#[test]
fn operator_deregistration_valid() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let operator_0 = 0;
        let operator_1 = 1;
        let operator_2 = 2;

        assert!(<Oracle>::register_operator(RuntimeOrigin::root(), operator_1).is_ok());
        assert!(<Oracle>::deregister_operator(RuntimeOrigin::root(), operator_1).is_ok());
        assert_eq!(<Oracle>::num_registered_operators(), 0);

        assert_eq!(last_event(), OracleEvent::OperatorDeregistered(operator_1));

        assert!(<Oracle>::register_operator(RuntimeOrigin::root(), operator_2).is_ok());
        assert!(<Oracle>::register_operator(RuntimeOrigin::root(), operator_0).is_ok());
        assert!(<Oracle>::register_operator(RuntimeOrigin::root(), operator_1).is_ok());
        assert!(<Oracle>::deregister_operator(RuntimeOrigin::root(), operator_0).is_ok());
        assert_eq!(<Oracle>::num_registered_operators(), 2);
        assert!(<Oracle>::operator(operator_1).is_some());
        assert!(<Oracle>::operator(operator_2).is_some());

        assert_eq!(last_event(), OracleEvent::OperatorDeregistered(operator_0));
    });
}

#[test]
fn operator_activation_and_deactivation() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let operator_0 = 0;
        let operator_1 = 1;
        let operator_2 = 2;
        let operator_3 = 3;
        let operator_4 = 4;

        <Oracle>::register_operator(RuntimeOrigin::root(), operator_0).unwrap();
        <Oracle>::register_operator(RuntimeOrigin::root(), operator_1).unwrap();
        <Oracle>::register_operator(RuntimeOrigin::root(), operator_2).unwrap();

        <Oracle>::activate_operator(RuntimeOrigin::signed(operator_2)).unwrap();
        <Oracle>::activate_operator(RuntimeOrigin::signed(operator_0)).unwrap();
        <Oracle>::activate_operator(RuntimeOrigin::signed(operator_1)).unwrap();

        assert_eq!(last_event(), OracleEvent::OperatorActivated(operator_1));
        assert_eq!(
            <Oracle>::active_operators(),
            vec![operator_0, operator_1, operator_2]
        );
        assert_eq!(<Oracle>::num_registered_operators(), 3);

        // deactivate operator_0
        <Oracle>::deactivate_operator(RuntimeOrigin::signed(operator_0)).unwrap();

        assert_eq!(last_event(), OracleEvent::OperatorDeactivated(operator_0));
        assert_eq!(<Oracle>::active_operators(), vec![operator_1, operator_2]);
        assert_eq!(<Oracle>::num_registered_operators(), 3);

        // activate all registered operators (reactivate operator_0
        <Oracle>::register_operator(RuntimeOrigin::root(), operator_3).unwrap();
        assert_eq!(<Oracle>::num_registered_operators(), 4);
        <Oracle>::activate_operator(RuntimeOrigin::signed(operator_3)).unwrap();
        <Oracle>::activate_operator(RuntimeOrigin::signed(operator_0)).unwrap();
        assert_eq!(
            <Oracle>::active_operators(),
            vec![operator_0, operator_1, operator_2, operator_3]
        );

        // not yet registered operator tries to activate
        assert_eq!(
            error_msg(<Oracle>::activate_operator(RuntimeOrigin::signed(operator_4)).unwrap_err()),
            "UnknownOperator"
        );

        // deregister an activated operator
        <Oracle>::deregister_operator(RuntimeOrigin::root(), operator_1).unwrap();
        assert_eq!(<Oracle>::num_registered_operators(), 3);
        assert_eq!(
            <Oracle>::active_operators(),
            vec![operator_0, operator_2, operator_3]
        );
        assert_eq!(
            error_msg(
                <Oracle>::deactivate_operator(RuntimeOrigin::signed(operator_1)).unwrap_err()
            ),
            "UnknownOperator"
        );
        // deregister a deactivated operator
        <Oracle>::deactivate_operator(RuntimeOrigin::signed(operator_2)).unwrap();
        assert_eq!(<Oracle>::active_operators(), vec![operator_0, operator_3]);
        assert_eq!(<Oracle>::num_registered_operators(), 3);
        <Oracle>::deregister_operator(RuntimeOrigin::root(), operator_2).unwrap();
        assert_eq!(<Oracle>::num_registered_operators(), 2);
        // deregistered tries to re-activate again
        assert_eq!(
            error_msg(<Oracle>::activate_operator(RuntimeOrigin::signed(operator_2)).unwrap_err()),
            "UnknownOperator"
        );
        // register a new operator
        <Oracle>::register_operator(RuntimeOrigin::root(), operator_4).unwrap();
        assert_eq!(<Oracle>::num_registered_operators(), 3);
        <Oracle>::register_operator(RuntimeOrigin::root(), operator_2).unwrap();
        assert_eq!(<Oracle>::num_registered_operators(), 4);
        <Oracle>::activate_operator(RuntimeOrigin::signed(operator_4)).unwrap();
        <Oracle>::activate_operator(RuntimeOrigin::signed(operator_2)).unwrap();
        assert_eq!(
            <Oracle>::active_operators(),
            vec![operator_0, operator_2, operator_3, operator_4]
        );
    });
}

#[test]
fn initiate_requests_valid() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let callback = MockCallback::test();
        let fee = minimum_fee();
        let parameters = ("a", "b");
        let data = parameters.encode();
        let result = vec![10, 0, 0, 0];
        let request_id = 0;

        <Oracle>::register_operator(RuntimeOrigin::root(), ACCOUNT_0).unwrap();
        assert_eq!(last_event(), OracleEvent::OperatorRegistered(ACCOUNT_0));
        <Oracle>::activate_operator(RuntimeOrigin::signed(ACCOUNT_0)).unwrap();
        assert_eq!(last_event(), OracleEvent::OperatorActivated(ACCOUNT_0));

        <Oracle>::initiate_request(
            RuntimeOrigin::signed(ACCOUNT_1),
            callback,
            data.clone(),
            fee,
        )
        .unwrap();

        assert_eq!(
            last_event(),
            OracleEvent::OracleRequest {
                request_id,
                operator: ACCOUNT_0,
                callback,
                fee,
            }
        );

        let r = <(Vec<u8>, Vec<u8>)>::decode(&mut &data[..]).unwrap();
        assert_eq!(parameters.0, std::str::from_utf8(&r.0).unwrap());
        assert_eq!(parameters.1, std::str::from_utf8(&r.1).unwrap());

        <Oracle>::callback(RuntimeOrigin::signed(ACCOUNT_0), request_id, result.clone()).unwrap();

        assert_eq!(
            last_event(),
            OracleEvent::OracleAnswer {
                request_id,
                operator: ACCOUNT_0,
                fee,
                result,
            }
        );
    });
}

#[test]
fn linear_request_delegation() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let operator = 9;
        let operator_0 = 10;
        let operator_1 = 11;
        let operator_2 = 12;
        let operator_3 = 13;
        let data = vec![];
        let callback = MockCallback::test();
        let fee = minimum_fee();
        let mut request_id = 0;

        <Oracle>::register_operator(RuntimeOrigin::root(), operator).unwrap();
        <Oracle>::register_operator(RuntimeOrigin::root(), operator_0).unwrap();
        <Oracle>::register_operator(RuntimeOrigin::root(), operator_1).unwrap();
        <Oracle>::register_operator(RuntimeOrigin::root(), operator_2).unwrap();
        assert_eq!(
            error_msg(<Oracle>::register_operator(RuntimeOrigin::root(), operator_3).unwrap_err()),
            "MaxOperatorsRegistered"
        );
        assert_eq!(
            <Oracle>::num_registered_operators(),
            <TestRuntime as pallet_oracle::Config>::MaxOperators::get()
        );
        <Oracle>::deregister_operator(RuntimeOrigin::root(), operator).unwrap();
        <Oracle>::register_operator(RuntimeOrigin::root(), operator_3).unwrap();

        // activate operators
        <Oracle>::activate_operator(RuntimeOrigin::signed(operator_0)).unwrap();
        <Oracle>::activate_operator(RuntimeOrigin::signed(operator_1)).unwrap();
        <Oracle>::activate_operator(RuntimeOrigin::signed(operator_2)).unwrap();
        <Oracle>::activate_operator(RuntimeOrigin::signed(operator_3)).unwrap();

        <Oracle>::initiate_request(
            RuntimeOrigin::signed(ACCOUNT_0),
            callback,
            data.clone(),
            fee,
        )
        .unwrap();

        assert_eq!(
            last_event(),
            OracleEvent::OracleRequest {
                request_id,
                operator: operator_0,
                callback,
                fee,
            }
        );
        request_id += 1;

        <Oracle>::initiate_request(
            RuntimeOrigin::signed(ACCOUNT_0),
            callback,
            data.clone(),
            fee,
        )
        .unwrap();

        assert_eq!(
            last_event(),
            OracleEvent::OracleRequest {
                request_id,
                operator: operator_1,
                callback,
                fee,
            }
        );
        request_id += 1;

        <Oracle>::initiate_request(
            RuntimeOrigin::signed(ACCOUNT_0),
            callback,
            data.clone(),
            fee,
        )
        .unwrap();

        assert_eq!(
            last_event(),
            OracleEvent::OracleRequest {
                request_id,
                operator: operator_2,
                callback,
                fee,
            }
        );
        request_id += 1;

        <Oracle>::initiate_request(
            RuntimeOrigin::signed(ACCOUNT_0),
            callback,
            data.clone(),
            fee,
        )
        .unwrap();

        assert_eq!(
            last_event(),
            OracleEvent::OracleRequest {
                request_id,
                operator: operator_3,
                callback,
                fee,
            }
        );
        request_id += 1;

        <Oracle>::initiate_request(
            RuntimeOrigin::signed(ACCOUNT_0),
            callback,
            data.clone(),
            fee,
        )
        .unwrap();

        assert_eq!(
            last_event(),
            OracleEvent::OracleRequest {
                request_id,
                operator: operator_0,
                callback,
                fee,
            }
        );

        request_id += 1;

        // operator_1, and operator_2 deactivates
        <Oracle>::deactivate_operator(RuntimeOrigin::signed(operator_1)).unwrap();
        <Oracle>::deactivate_operator(RuntimeOrigin::signed(operator_2)).unwrap();
        <Oracle>::initiate_request(
            RuntimeOrigin::signed(ACCOUNT_0),
            callback,
            data.clone(),
            fee,
        )
        .unwrap();
        assert_eq!(
            last_event(),
            OracleEvent::OracleRequest {
                request_id,
                operator: operator_3,
                callback,
                fee,
            }
        );

        request_id += 1;

        // operator_0 is deregistered by root
        <Oracle>::deregister_operator(RuntimeOrigin::root(), operator_0).unwrap();
        assert_eq!(<Oracle>::active_operators(), vec![operator_3]);
        <Oracle>::initiate_request(
            RuntimeOrigin::signed(ACCOUNT_0),
            callback,
            data.clone(),
            fee,
        )
        .unwrap();
        assert_eq!(
            last_event(),
            OracleEvent::OracleRequest {
                request_id,
                operator: operator_3,
                callback,
                fee,
            }
        );

        request_id += 1;

        <Oracle>::initiate_request(RuntimeOrigin::signed(ACCOUNT_0), callback, data, fee).unwrap();
        assert_eq!(
            last_event(),
            OracleEvent::OracleRequest {
                request_id,
                operator: operator_3,
                callback,
                fee,
            }
        );
    });
}

#[test]
fn initiate_requests_invalid_insufficient_fee() {
    new_test_ext().execute_with(|| {
        <Oracle>::register_operator(RuntimeOrigin::root(), ACCOUNT_0).unwrap();
        <Oracle>::activate_operator(RuntimeOrigin::signed(ACCOUNT_0)).unwrap();
        let error = <Oracle>::initiate_request(
            RuntimeOrigin::signed(ACCOUNT_1),
            MockCallback::test(),
            vec![],
            minimum_fee() - 1,
        )
        .unwrap_err();

        assert_eq!(error_msg(error), "InsufficientFee");
    });
}

#[test]
fn initiate_requests_invalid_insufficient_balance_for_fee() {
    new_test_ext().execute_with(|| {
        <Oracle>::register_operator(RuntimeOrigin::root(), ACCOUNT_0).unwrap();
        <Oracle>::activate_operator(RuntimeOrigin::signed(ACCOUNT_0)).unwrap();
        let error = <Oracle>::initiate_request(
            RuntimeOrigin::signed(ACCOUNT_1),
            MockCallback::test(),
            vec![],
            GENESIS_BALANCE + 1,
        )
        .unwrap_err();
        assert_eq!(error_msg(error), "InsufficientBalance");
    });
}

#[test]
fn initiate_requests_invalid_wrong_operator() {
    new_test_ext().execute_with(|| {
        <Oracle>::register_operator(RuntimeOrigin::root(), ACCOUNT_0).unwrap();
        <Oracle>::activate_operator(RuntimeOrigin::signed(ACCOUNT_0)).unwrap();
        <Oracle>::initiate_request(
            RuntimeOrigin::signed(ACCOUNT_1),
            MockCallback::test(),
            vec![],
            minimum_fee(),
        )
        .unwrap();
        let error = <Oracle>::callback(RuntimeOrigin::signed(99), 0, vec![1]).unwrap_err();
        assert_eq!(error_msg(error), "WrongOperator");
    });
}

#[test]
fn unknown_callback() {
    new_test_ext().execute_with(|| {
        <Oracle>::register_operator(RuntimeOrigin::root(), ACCOUNT_0).unwrap();
        <Oracle>::activate_operator(RuntimeOrigin::signed(ACCOUNT_0)).unwrap();
        <Oracle>::initiate_request(
            RuntimeOrigin::signed(ACCOUNT_1),
            MockCallback::test(),
            vec![],
            minimum_fee(),
        )
        .unwrap();
        // Sending an empty result in this test runtime environment causes
        // MockCallback to return None for the `with_result` call. The
        // resulting None will trigger the UnknownCallback error. Note, that
        // this is a very specific implementation of `CallbackWithParameter`
        // that was tailored for this edge case.
        let error = <Oracle>::callback(RuntimeOrigin::signed(ACCOUNT_0), 0, vec![]).unwrap_err();
        assert_eq!(error_msg(error), "UnknownCallback");
    });
}

#[test]
fn kill_request() {
    new_test_ext().execute_with(|| {
        let request_id = 0;

        <Oracle>::register_operator(RuntimeOrigin::root(), ACCOUNT_0).unwrap();
        <Oracle>::activate_operator(RuntimeOrigin::signed(ACCOUNT_0)).unwrap();
        <Oracle>::initiate_request(
            RuntimeOrigin::signed(ACCOUNT_1),
            MockCallback::test(),
            vec![],
            minimum_fee(),
        )
        .unwrap();

        <Oracle as OnFinalize<u64>>::on_finalize(
            <TestRuntime as pallet_oracle::Config>::ValidityPeriod::get() - 1,
        );

        assert!(<Oracle>::request(request_id).is_some());

        <Oracle as OnFinalize<u64>>::on_finalize(
            <TestRuntime as pallet_oracle::Config>::ValidityPeriod::get() + 1,
        );
        // Request has been killed, too old
        // Unknown request error
        let error =
            <Oracle>::callback(RuntimeOrigin::signed(1), request_id, 10.encode()).unwrap_err();
        assert_eq!(error_msg(error), "UnknownRequest");
        assert!(<Oracle>::request(request_id).is_none());
    });
}
