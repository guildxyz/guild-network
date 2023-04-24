use crate::mock::*;

#[test]
fn invalid_transactions_fail() {
    new_test_ext().execute_with(|| {
        let failing_transactions = vec![
            (<GuildIdentity>::register(RuntimeOrigin::none()), "BadOrigin")
        ];
    });
}
