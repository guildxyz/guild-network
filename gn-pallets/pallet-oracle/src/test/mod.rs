mod helpers;
mod initiate;
mod register;

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
                <Oracle>::callback(RuntimeOrigin::signed(0), 0, 0),
                DispatchError::BadOrigin,
            ),
            (
                <Oracle>::callback(RuntimeOrigin::none(), 0, 0),
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
                <Oracle>::callback(RuntimeOrigin::root(), 0, 0),
                OracleError::UnknownRequest.into(),
            ),
        ];

        for (tx, error) in failing_transactions {
            assert_noop!(tx, error);
        }
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
            <Oracle>::callback(RuntimeOrigin::root(), ACCOUNT_0, request_id),
            OracleError::UnknownRequest
        );
        assert!(<Oracle>::request(request_id).is_none());
    });
}
