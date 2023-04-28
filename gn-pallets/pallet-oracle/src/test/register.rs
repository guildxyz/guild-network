use super::*;

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
