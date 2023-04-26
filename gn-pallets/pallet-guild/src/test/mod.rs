//mod guild_and_role;
mod helpers;
//mod join_and_leave;

use crate::mock::*;
use frame_support::dispatch::DispatchError;
use frame_support::{assert_noop, assert_ok};
use gn_common::filter::{Guild as GuildFilter, Logic as FilterLogic};
use gn_common::merkle::Proof as MerkleProof;
use helpers::*;

#[test]
fn signer_checks() {
    new_test_ext().execute_with(|| {
        let guild = [0u8; 32];
        let role = [1u8; 32];
        let filter = GuildFilter {
            name: guild,
            role: Some(role),
        };
        let failing_transactions = vec![
            <Guild>::join_free_role(RuntimeOrigin::root(), guild, role),
            <Guild>::join_free_role(RuntimeOrigin::none(), guild, role),
            <Guild>::join_child_role(RuntimeOrigin::root(), guild, role),
            <Guild>::join_child_role(RuntimeOrigin::none(), guild, role),
            <Guild>::join_role_with_allowlist(
                RuntimeOrigin::root(),
                guild,
                role,
                MerkleProof::new(&[&[1], &[2]], 0),
            ),
            <Guild>::join_role_with_allowlist(
                RuntimeOrigin::none(),
                guild,
                role,
                MerkleProof::new(&[&[1], &[2]], 0),
            ),
            <Guild>::join_unfiltered_role(RuntimeOrigin::root(), guild, role),
            <Guild>::join_unfiltered_role(RuntimeOrigin::none(), guild, role),
            <Guild>::leave(RuntimeOrigin::root(), guild, role),
            <Guild>::leave(RuntimeOrigin::none(), guild, role),
            <Guild>::request_access_check(RuntimeOrigin::root(), 0, guild, role),
            <Guild>::request_access_check(RuntimeOrigin::none(), 0, guild, role),
            <Guild>::create_guild(RuntimeOrigin::root(), guild, vec![]),
            <Guild>::create_guild(RuntimeOrigin::none(), guild, vec![]),
            <Guild>::create_free_role(RuntimeOrigin::root(), guild, role),
            <Guild>::create_free_role(RuntimeOrigin::none(), guild, role),
            <Guild>::create_role_with_allowlist(
                RuntimeOrigin::root(),
                guild,
                role,
                vec![[0u8; 32], [1u8; 32]],
                FilterLogic::And,
                None,
            ),
            <Guild>::create_role_with_allowlist(
                RuntimeOrigin::none(),
                guild,
                role,
                vec![[0u8; 32], [1u8; 32]],
                FilterLogic::And,
                None,
            ),
            <Guild>::create_child_role(
                RuntimeOrigin::root(),
                guild,
                role,
                filter,
                FilterLogic::Or,
                None,
            ),
            <Guild>::create_child_role(
                RuntimeOrigin::none(),
                guild,
                role,
                filter,
                FilterLogic::Or,
                None,
            ),
            <Guild>::create_unfiltered_role(RuntimeOrigin::root(), guild, role, (vec![], vec![])),
            <Guild>::create_unfiltered_role(RuntimeOrigin::none(), guild, role, (vec![], vec![])),
            <Guild>::sudo_remove(RuntimeOrigin::signed(0), 0, guild, role),
            <Guild>::sudo_remove(RuntimeOrigin::none(), 0, guild, role),
        ];

        for tx in failing_transactions {
            assert_noop!(tx, DispatchError::BadOrigin);
        }
    });
}
