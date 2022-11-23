use crate as pallet_test_caller;
use codec::{Decode, Encode};
use frame_support::traits::OnFinalize;
use test_runtime::test_runtime;

test_runtime!(TestCaller, pallet_test_caller);

pub fn last_event() -> Event {
    System::events()
        .into_iter()
        .filter_map(|e| {
            if let Event::Chainlink(inner) = e.event {
                Some(Event::Chainlink(inner))
            } else {
                None
            }
        })
        .last()
        .unwrap()
}

fn get_minimum_fee() -> u64 {
    <TestRuntime as pallet_chainlink::Config>::MinimumFee::get() as u64
}

#[test]
fn operator_registration_valid() {
    new_test_runtime().execute_with(|| {
        // This is required for some reason otherwise the last_event() method fails
        System::set_block_number(1);

        assert!(<Chainlink>::operators().is_empty());
        assert!(<Chainlink>::register_operator(Origin::signed(1)).is_ok());
        assert_eq!(
            last_event(),
            Event::Chainlink(pallet_chainlink::Event::OperatorRegistered(1))
        );
        assert_eq!(<Chainlink>::operators(), vec![1]);
    });
}

#[test]
fn operator_registration_invalid_operator_already_registered() {
    new_test_runtime().execute_with(|| {
        assert!(<Chainlink>::register_operator(Origin::signed(1)).is_ok());
        assert_eq!(<Chainlink>::operators(), vec![1]);

        // Operator already registered error
        assert!(<Chainlink>::register_operator(Origin::signed(1)).is_err());
        assert_eq!(<Chainlink>::operators(), vec![1]);
    });
}

#[test]
fn operator_unregistration_valid() {
    new_test_runtime().execute_with(|| {
        // This is required for some reason otherwise the last_event() method fails
        System::set_block_number(1);

        assert!(<Chainlink>::register_operator(Origin::signed(1)).is_ok());
        assert!(<Chainlink>::deregister_operator(Origin::signed(1)).is_ok());
        assert!(<Chainlink>::operators().is_empty());

        assert_eq!(
            last_event(),
            Event::Chainlink(pallet_chainlink::Event::OperatorDeregistered(1))
        );
    });
}

#[test]
fn operator_unregistration_invalid_unknown_operator() {
    new_test_runtime().execute_with(|| {
        // Unknown operator error
        assert!(<Chainlink>::deregister_operator(Origin::signed(1)).is_err());
        assert!(<Chainlink>::operators().is_empty());
    });
}

#[test]
fn initiate_requests_valid() {
    new_test_runtime().execute_with(|| {
        System::set_block_number(1);
        let operator = 1;
        let requester = 2;
        let callback = pallet_test_caller::Call::<TestRuntime>::callback { result: vec![] };
        let fee = get_minimum_fee();

        assert!(<Chainlink>::register_operator(Origin::signed(operator)).is_ok());
        assert_eq!(
            last_event(),
            Event::Chainlink(pallet_chainlink::Event::OperatorRegistered(operator))
        );

        let parameters = ("a", "b");
        let data = parameters.encode();
        assert!(<Chainlink>::initiate_request(
            Origin::signed(requester),
            callback.clone(),
            data.clone(),
            fee,
        )
        .is_ok());

        let request_id = 0;
        assert_eq!(
            last_event(),
            Event::Chainlink(pallet_chainlink::Event::OracleRequest {
                request_id,
                operator,
                callback,
                fee,
            })
        );

        let r = <(Vec<u8>, Vec<u8>)>::decode(&mut &data[..]).unwrap().0;
        assert_eq!("a", std::str::from_utf8(&r).unwrap());

        let result: u64 = 10;
        assert!(<Chainlink>::callback(Origin::signed(operator), request_id, result.encode()).is_ok());

        assert_eq!(
            last_event(),
            Event::Chainlink(pallet_chainlink::Event::OracleAnswer {
                request_id,
                operator,
                fee,
            })
        );

        assert_eq!(
            <pallet_test_caller::OracleAnswer<TestRuntime>>::get(),
            result
        );
    });
}

#[test]
fn linear_request_delegation() {
    new_test_runtime().execute_with(|| {
        System::set_block_number(1);

        let signer = 1;
        let operator_0 = 4;
        let operator_1 = 2;
        let operator_2 = 3;
        let operator_3 = 5;
        let data = vec![];
        let callback = pallet_test_caller::Call::<TestRuntime>::callback { result: vec![] };
        let fee = get_minimum_fee();
        let mut request_id = 0;

        assert!(<Chainlink>::register_operator(Origin::signed(operator_0)).is_ok());
        assert!(<Chainlink>::register_operator(Origin::signed(operator_1)).is_ok());
        assert!(<Chainlink>::register_operator(Origin::signed(operator_2)).is_ok());
        assert!(<Chainlink>::register_operator(Origin::signed(operator_3)).is_ok());

        assert!(<Chainlink>::initiate_request(
            Origin::signed(signer),
            callback.clone(),
            data.clone(),
            fee,
        )
        .is_ok());

        assert_eq!(
            last_event(),
            Event::Chainlink(pallet_chainlink::Event::OracleRequest {
                request_id,
                operator: operator_0,
                callback: callback.clone(),
                fee,
            })
        );
        request_id += 1;

        assert!(<Chainlink>::initiate_request(
            Origin::signed(signer),
            callback.clone(),
            data.clone(),
            fee,
        )
        .is_ok());

        assert_eq!(
            last_event(),
            Event::Chainlink(pallet_chainlink::Event::OracleRequest {
                request_id,
                operator: operator_1,
                callback: callback.clone(),
                fee,
            })
        );
        request_id += 1;

        assert!(<Chainlink>::initiate_request(
            Origin::signed(signer),
            callback.clone(),
            data.clone(),
            fee,
        )
        .is_ok());

        assert_eq!(
            last_event(),
            Event::Chainlink(pallet_chainlink::Event::OracleRequest {
                request_id,
                operator: operator_2,
                callback: callback.clone(),
                fee,
            })
        );
        request_id += 1;

        assert!(<Chainlink>::initiate_request(
            Origin::signed(signer),
            callback.clone(),
            data.clone(),
            fee,
        )
        .is_ok());

        assert_eq!(
            last_event(),
            Event::Chainlink(pallet_chainlink::Event::OracleRequest {
                request_id,
                operator: operator_3,
                callback: callback.clone(),
                fee,
            })
        );
        request_id += 1;

        assert!(<Chainlink>::initiate_request(
            Origin::signed(signer),
            callback.clone(),
            data.clone(),
            fee,
        )
        .is_ok());

        assert_eq!(
            last_event(),
            Event::Chainlink(pallet_chainlink::Event::OracleRequest {
                request_id,
                operator: operator_0,
                callback,
                fee,
            })
        );
    });
}

#[test]
fn initiate_requests_invalid_unknown_operator() {
    new_test_runtime().execute_with(|| {
        // No operator registered error
        assert!(<Chainlink>::initiate_request(
            Origin::signed(2),
            pallet_test_caller::Call::<TestRuntime>::callback { result: vec![] },
            vec![],
            get_minimum_fee(),
        )
        .is_err());
    });
}

#[test]
fn initiate_requests_invalid_insufficient_fee() {
    new_test_runtime().execute_with(|| {
        assert!(<Chainlink>::register_operator(Origin::signed(1)).is_ok());
        // Insufficient fee error
        assert!(<Chainlink>::initiate_request(
            Origin::signed(2),
            pallet_test_caller::Call::<TestRuntime>::callback { result: vec![] },
            vec![],
            get_minimum_fee() - 1,
        )
        .is_err());
    });
}

#[test]
fn initiate_requests_invalid_insufficient_balance_for_fee() {
    new_test_runtime().execute_with(|| {
        assert!(<Chainlink>::register_operator(Origin::signed(1)).is_ok());

        // Insufficient balance error (System error)
        assert!(<Chainlink>::initiate_request(
            Origin::signed(2),
            pallet_test_caller::Call::<TestRuntime>::callback { result: vec![] },
            vec![],
            GENESIS_BALANCE + 1,
        )
        .is_err());
    });
}

#[test]
fn initiate_requests_invalid_wrong_operator() {
    new_test_runtime().execute_with(|| {
        assert!(<Chainlink>::register_operator(Origin::signed(1)).is_ok());
        assert!(<Chainlink>::initiate_request(
            Origin::signed(2),
            pallet_test_caller::Call::<TestRuntime>::callback { result: vec![] },
            vec![],
            get_minimum_fee(),
        )
        .is_ok());
        // Wrong operator error
        assert!(<Chainlink>::callback(Origin::signed(3), 0, 10.encode()).is_err());
    });
}

#[test]
fn callback_invalid_unknown_request() {
    new_test_runtime().execute_with(|| {
        // Unknown request error
        assert!(<Chainlink>::callback(Origin::signed(1), 0, 10.encode()).is_err());
    });
}

#[test]
fn kill_request() {
    new_test_runtime().execute_with(|| {
        assert!(<Chainlink>::register_operator(Origin::signed(1)).is_ok());
        assert!(<Chainlink>::initiate_request(
            Origin::signed(2),
            pallet_test_caller::Call::<TestRuntime>::callback { result: vec![] },
            vec![],
            get_minimum_fee(),
        )
        .is_ok());

        <Chainlink as OnFinalize<u64>>::on_finalize(
            <TestRuntime as pallet_chainlink::Config>::ValidityPeriod::get() - 1,
        );

        assert!(<Chainlink>::request(0).is_some());

        <Chainlink as OnFinalize<u64>>::on_finalize(
            <TestRuntime as pallet_chainlink::Config>::ValidityPeriod::get() + 1,
        );
        // Request has been killed, too old
        // Unknown request error
        assert!(<Chainlink>::callback(Origin::signed(1), 0, 10.encode()).is_err());
        assert!(<Chainlink>::request(0).is_none());
    });
}
