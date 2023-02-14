use super::*;

#[test]
fn join_and_leave_free_role() {
    new_test_ext().execute_with(|| {
        init_chain();
        let owner = 0;
        let user = 1;
        let guild_name = [0u8; 32];
        let role_name = [0u8; 32];
        let invalid_name = [100u8; 32];
        dummy_guild(owner, guild_name);
        <Guild>::create_free_role(RuntimeOrigin::signed(owner), guild_name, role_name).unwrap();

        let failing_transactions = vec![
            (
                <Guild>::join(RuntimeOrigin::none(), guild_name, role_name, None),
                "BadOrigin",
            ),
            (
                <Guild>::join(RuntimeOrigin::root(), guild_name, role_name, None),
                "BadOrigin",
            ),
            (
                <Guild>::join(RuntimeOrigin::signed(user), invalid_name, role_name, None),
                "GuildDoesNotExist",
            ),
            (
                <Guild>::join(RuntimeOrigin::signed(user), guild_name, invalid_name, None),
                "RoleDoesNotExist",
            ),
            (
                <Guild>::join(RuntimeOrigin::signed(user), guild_name, role_name, None),
                "UserNotRegistered",
            ),
        ];

        for (tx, raw_error_msg) in failing_transactions {
            assert_eq!(error_msg(tx.unwrap_err()), raw_error_msg);
        }
    });
}
// TODO copy stuff from github (actually joining roles)
