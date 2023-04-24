mod helpers;
use helpers::*;

use crate::mock::*;
use frame_support::traits::OnFinalize;
use frame_support::{assert_noop, assert_ok};
use parity_scale_codec::{Decode, Encode};

#[test]
fn invalid_transactions_fail() {
    new_test_ext().execute_with(|| {
        let failing_transactions = vec![
            (
                <Oracle>::register_operator(RuntimeOrigin::none(), 0),
                DispatchError::BadOrigin,
            ),
            (
                <Oracle>::register_operator(RuntimeOrigin::signed(0), 0),
                DispatchError::BadOrigin,
            ),
            (
                <Oracle>::deregister_operator(RuntimeOrigin::none(), 0),
                DispatchError::BadOrigin,
            ),
            (
                <Oracle>::deregister_operator(RuntimeOrigin::signed(1), 0),
                DispatchError::BadOrigin,
            ),
            (
                <Oracle>::activate_operator(RuntimeOrigin::signed(1)),
                OracleError::UnknownOperator.into(),
            ),
            (
                <Oracle>::deactivate_operator(RuntimeOrigin::signed(1)),
                OracleError::UnknownOperator.into(),
            ),
            (
                <Oracle>::deregister_operator(RuntimeOrigin::root(), 1),
                OracleError::UnknownOperator.into(),
            ),
            (
                <Oracle>::initiate_request(RuntimeOrigin::signed(1), 0, vec![], minimum_fee()),
                OracleError::NoActiveOperators.into(),
            ),
            (
                <Oracle>::callback(RuntimeOrigin::signed(ACCOUNT_0), 0),
                OracleError::UnknownRequest.into(),
            ),
        ];

        for (tx_result, error) in failing_transactions {
            assert_noop!(tx_result, error);
        }
    });
}

#[test]
fn operator_registration_valid() {
    new_test_ext().execute_with(|| {
        // This is required for some reason otherwise the last_event() method fails
        let operator = 1;
        assert_eq!(<Oracle>::num_registered_operators(), 0);
        assert_ok!(<Oracle>::register_operator(RuntimeOrigin::root(), operator));
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
        assert_ok!(<Oracle>::register_operator(
            RuntimeOrigin::root(),
            operator_1
        ));
        assert!(<Oracle>::operator(operator_1).is_some());
        assert_eq!(<Oracle>::num_registered_operators(), 1);

        // Operator already registered error
        assert_noop!(
            <Oracle>::register_operator(RuntimeOrigin::root(), operator_1),
            OracleError::OperatorAlreadyRegistered
        );
        assert!(<Oracle>::operator(operator_1).is_some());
        assert_eq!(<Oracle>::num_registered_operators(), 1);

        assert_ok!(<Oracle>::register_operator(
            RuntimeOrigin::root(),
            operator_2
        ));
        assert!(<Oracle>::operator(operator_1).is_some());
        assert!(<Oracle>::operator(operator_2).is_some());
        assert_eq!(<Oracle>::num_registered_operators(), 2);
    });
}

#[test]
fn operator_deregistration_valid() {
    new_test_ext().execute_with(|| {
        let operator_0 = 0;
        let operator_1 = 1;
        let operator_2 = 2;

        assert_ok!(<Oracle>::register_operator(
            RuntimeOrigin::root(),
            operator_1
        ));
        assert_ok!(<Oracle>::deregister_operator(
            RuntimeOrigin::root(),
            operator_1
        ));
        assert_eq!(<Oracle>::num_registered_operators(), 0);

        assert_eq!(last_event(), OracleEvent::OperatorDeregistered(operator_1));

        assert_ok!(<Oracle>::register_operator(
            RuntimeOrigin::root(),
            operator_2
        ));
        assert_ok!(<Oracle>::register_operator(
            RuntimeOrigin::root(),
            operator_0
        ));
        assert_ok!(<Oracle>::register_operator(
            RuntimeOrigin::root(),
            operator_1
        ));
        assert_ok!(<Oracle>::deregister_operator(
            RuntimeOrigin::root(),
            operator_0
        ));
        assert_eq!(<Oracle>::num_registered_operators(), 2);
        assert!(<Oracle>::operator(operator_1).is_some());
        assert!(<Oracle>::operator(operator_2).is_some());

        assert_eq!(last_event(), OracleEvent::OperatorDeregistered(operator_0));
    });
}

#[test]
fn operator_activation_and_deactivation() {
    new_test_ext().execute_with(|| {
        let operator_0 = 0;
        let operator_1 = 1;
        let operator_2 = 2;
        let operator_3 = 3;
        let operator_4 = 4;

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

        assert_ok!(<Oracle>::activate_operator(RuntimeOrigin::signed(
            operator_2
        )));
        assert_ok!(<Oracle>::activate_operator(RuntimeOrigin::signed(
            operator_0
        )));
        assert_ok!(<Oracle>::activate_operator(RuntimeOrigin::signed(
            operator_1
        )));

        assert_eq!(last_event(), OracleEvent::OperatorActivated(operator_1));
        assert_eq!(
            <Oracle>::active_operators(),
            vec![operator_0, operator_1, operator_2]
        );
        assert_eq!(<Oracle>::num_registered_operators(), 3);

        // deactivate operator_0
        assert_ok!(<Oracle>::deactivate_operator(RuntimeOrigin::signed(
            operator_0
        )));

        assert_eq!(last_event(), OracleEvent::OperatorDeactivated(operator_0));
        assert_eq!(<Oracle>::active_operators(), vec![operator_1, operator_2]);
        assert_eq!(<Oracle>::num_registered_operators(), 3);

        // activate all registered operators (reactivate operator_0
        assert_ok!(<Oracle>::register_operator(
            RuntimeOrigin::root(),
            operator_3
        ));
        assert_eq!(<Oracle>::num_registered_operators(), 4);
        assert_ok!(<Oracle>::activate_operator(RuntimeOrigin::signed(
            operator_3
        )));
        assert_ok!(<Oracle>::activate_operator(RuntimeOrigin::signed(
            operator_0
        )));
        assert_eq!(
            <Oracle>::active_operators(),
            vec![operator_0, operator_1, operator_2, operator_3]
        );

        // not yet registered operator tries to activate
        assert_noop!(
            <Oracle>::activate_operator(RuntimeOrigin::signed(operator_4)),
            OracleError::UnknownOperator
        );

        // deregister an activated operator
        assert_ok!(<Oracle>::deregister_operator(
            RuntimeOrigin::root(),
            operator_1
        ));
        assert_eq!(<Oracle>::num_registered_operators(), 3);
        assert_eq!(
            <Oracle>::active_operators(),
            vec![operator_0, operator_2, operator_3]
        );
        assert_noop!(
            <Oracle>::deactivate_operator(RuntimeOrigin::signed(operator_1)),
            OracleError::UnknownOperator
        );
        // deregister a deactivated operator
        assert_ok!(<Oracle>::deactivate_operator(RuntimeOrigin::signed(
            operator_2
        )));
        assert_eq!(<Oracle>::active_operators(), vec![operator_0, operator_3]);
        assert_eq!(<Oracle>::num_registered_operators(), 3);
        assert_ok!(<Oracle>::deregister_operator(
            RuntimeOrigin::root(),
            operator_2
        ));
        assert_eq!(<Oracle>::num_registered_operators(), 2);
        // deregistered tries to re-activate again
        assert_noop!(
            <Oracle>::activate_operator(RuntimeOrigin::signed(operator_2)),
            OracleError::UnknownOperator
        );
        // register a new operator
        assert_ok!(<Oracle>::register_operator(
            RuntimeOrigin::root(),
            operator_4
        ));
        assert_eq!(<Oracle>::num_registered_operators(), 3);
        assert_ok!(<Oracle>::register_operator(
            RuntimeOrigin::root(),
            operator_2
        ));
        assert_eq!(<Oracle>::num_registered_operators(), 4);
        assert_ok!(<Oracle>::activate_operator(RuntimeOrigin::signed(
            operator_4
        )));
        assert_ok!(<Oracle>::activate_operator(RuntimeOrigin::signed(
            operator_2
        )));
        assert_eq!(
            <Oracle>::active_operators(),
            vec![operator_0, operator_2, operator_3, operator_4]
        );
    });
}

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
            RuntimeOrigin::signed(ACCOUNT_0),
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
            <Oracle>::callback(RuntimeOrigin::signed(99), 0),
            OracleError::WrongOperator
        );
    });
}

#[test]
fn kill_request() {
    new_test_ext().execute_with(|| {
        let request_id = 0;
        let current_block = <System>::block_number();

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

        <Oracle as OnFinalize<u64>>::on_finalize(
            current_block + <TestRuntime as pallet_oracle::Config>::ValidityPeriod::get() - 1,
        );

        assert!(<Oracle>::request(request_id).is_some());

        <Oracle as OnFinalize<u64>>::on_finalize(
            current_block + <TestRuntime as pallet_oracle::Config>::ValidityPeriod::get() + 1,
        );
        // Request has been killed, too old Unknown request error
        assert_noop!(
            <Oracle>::callback(RuntimeOrigin::signed(ACCOUNT_0), request_id),
            OracleError::UnknownRequest
        );
        assert!(<Oracle>::request(request_id).is_none());
    });
}
