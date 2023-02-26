mod guild_and_role;
mod helpers;
mod join_and_leave;
mod register;

use helpers::*;

use crate::mock::*;
type AccountId = <TestRuntime as frame_system::Config>::AccountId;

use gn_common::{
    identity::{Identity, IdentityWithAuth},
    GuildName, RequestData,
};
use pallet_guild::Event as GuildEvent;
use parity_scale_codec::Encode;
use sp_runtime::DispatchError;

#[test]
fn callback_can_only_be_called_by_root() {
    new_test_ext().execute_with(|| {
        let register_no_access = dummy_answer(
            vec![u8::from(false)],
            0,
            RequestData::Register {
                identity_with_auth: IdentityWithAuth::Other(Identity::Other([0u8; 64]), [0u8; 64]),
                index: 0,
            },
        )
        .encode();

        let register_access = dummy_answer(
            vec![u8::from(true)],
            1,
            RequestData::Register {
                identity_with_auth: IdentityWithAuth::Other(Identity::Other([0u8; 64]), [0u8; 64]),
                index: <TestRuntime as pallet_guild::Config>::MaxIdentities::get(),
            },
        )
        .encode();

        let reqcheck_no_access = dummy_answer(
            vec![u8::from(false)],
            2,
            RequestData::ReqCheck {
                account: 1,
                guild_name: [0; 32],
                role_name: [1; 32],
            },
        )
        .encode();

        let reqcheck_access = dummy_answer(
            vec![u8::from(true)],
            3,
            RequestData::ReqCheck {
                account: 1,
                guild_name: [0; 32],
                role_name: [1; 32],
            },
        )
        .encode();

        let answer_with_empty_result = dummy_answer(
            vec![],
            4,
            RequestData::ReqCheck {
                account: 1,
                guild_name: [0; 32],
                role_name: [1; 32],
            },
        )
        .encode();

        let answer_with_too_long_result = dummy_answer(
            vec![1, 2],
            5,
            RequestData::ReqCheck {
                account: 1,
                guild_name: [0; 32],
                role_name: [1; 32],
            },
        )
        .encode();

        let test_data = vec![
            (
                <Guild>::callback(RuntimeOrigin::signed(1), vec![]),
                "BadOrigin",
            ),
            (
                <Guild>::callback(RuntimeOrigin::none(), vec![]),
                "BadOrigin",
            ),
            (
                <Guild>::callback(RuntimeOrigin::root(), vec![1]),
                "CodecError",
            ),
            (
                <Guild>::callback(RuntimeOrigin::root(), register_no_access),
                "AccessDenied",
            ),
            (
                <Guild>::callback(RuntimeOrigin::root(), register_access),
                "MaxIdentitiesExceeded",
            ),
            (
                <Guild>::callback(RuntimeOrigin::root(), reqcheck_no_access),
                "GuildDoesNotExist", // sanity checks precede access check
            ),
            (
                <Guild>::callback(RuntimeOrigin::root(), reqcheck_access),
                "GuildDoesNotExist",
            ),
            (
                <Guild>::callback(RuntimeOrigin::root(), answer_with_empty_result),
                "InvalidOracleAnswer",
            ),
            (
                <Guild>::callback(RuntimeOrigin::root(), answer_with_too_long_result),
                "InvalidOracleAnswer",
            ),
        ];

        for (call, raw_error) in test_data {
            assert_eq!(error_msg(call.unwrap_err()), raw_error);
        }
    });
}
