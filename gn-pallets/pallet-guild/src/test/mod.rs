mod helpers;
mod register;
mod role;

use helpers::*;

use crate::mock::*;
type AccountId = <TestRuntime as frame_system::Config>::AccountId;

use frame_support::traits::{OnFinalize, OnInitialize};
use gn_common::{
    identity::{Identity, IdentityWithAuth},
    Request, RequestData,
};
use pallet_guild::Event as GuildEvent;
use parity_scale_codec::{Decode, Encode};
use sp_runtime::DispatchError;

fn new_test_ext() -> sp_io::TestExternalities {
    frame_system::GenesisConfig::default()
        .build_storage::<TestRuntime>()
        .unwrap()
        .into()
}

#[test]
fn create_guild() {
    new_test_ext().execute_with(|| {
        init_chain();
        let signer = 4;
        let guild_name = [0u8; 32];

        new_guild(signer, guild_name);

        assert_eq!(last_event(), GuildEvent::GuildCreated(signer, guild_name));

        let guild_id = <Guild>::guild_id(guild_name).unwrap();
        let guild = <Guild>::guild(guild_id).unwrap();
        assert_eq!(guild.data.owner, signer);
        assert_eq!(guild.data.metadata, METADATA);

        let role_1_id = <Guild>::role_id(guild_id, ROLE_1).unwrap();
        let role_2_id = <Guild>::role_id(guild_id, ROLE_2).unwrap();
        let role_3_id = <Guild>::role_id(guild_id, ROLE_3).unwrap();
        let queried_role_1_data = <Guild>::role(role_1_id).unwrap();
        let queried_role_2_data = <Guild>::role(role_2_id).unwrap();
        let queried_role_3_data = <Guild>::role(role_3_id).unwrap();

        assert_eq!(queried_role_1_data.logic, LOGIC_1);
        assert_eq!(queried_role_2_data.logic, LOGIC_2);
        assert_eq!(queried_role_3_data.logic, LOGIC_3);

        assert_eq!(queried_role_1_data.requirements[0], REQ_1);
        assert_eq!(queried_role_1_data.requirements[1], REQ_2);
        assert_eq!(queried_role_1_data.requirements[2], REQ_3);

        assert_eq!(queried_role_2_data.requirements[0], REQ_1);
        assert_eq!(queried_role_2_data.requirements[1], REQ_2);
        assert_eq!(queried_role_2_data.requirements[2], REQ_3);

        assert_eq!(queried_role_3_data.requirements[0], REQ_1);
        assert_eq!(queried_role_3_data.requirements[1], REQ_2);
        assert_eq!(queried_role_3_data.requirements[2], REQ_3);
    });
}

#[test]
fn bound_checks_on_creating_guild() {
    new_test_ext().execute_with(|| {
        init_chain();
        let max_roles_per_guild =
            <TestRuntime as pallet_guild::Config>::MaxRolesPerGuild::get() as usize;
        let max_reqs_per_role =
            <TestRuntime as pallet_guild::Config>::MaxReqsPerRole::get() as usize;
        let max_serialized_req_len =
            <TestRuntime as pallet_guild::Config>::MaxSerializedReqLen::get() as usize;

        let test_data = vec![
            (
                vec![([0u8; 32], (vec![], vec![])); max_roles_per_guild + 1],
                "MaxRolesPerGuildExceeded",
            ),
            (
                vec![
                    ([0u8; 32], (vec![], vec![vec![]; max_reqs_per_role + 1]));
                    max_roles_per_guild
                ],
                "MaxReqsPerRoleExceeded",
            ),
            (
                vec![(
                    [0u8; 32],
                    (vec![], vec![vec![0; max_serialized_req_len + 1]]),
                )],
                "MaxSerializedReqLenExceeded",
            ),
        ];

        for (roles, raw_error) in test_data {
            let error = <Guild>::create_guild(RuntimeOrigin::signed(0), [0; 32], vec![], roles)
                .unwrap_err();
            assert_eq!(error_msg(error), raw_error);
        }
    });
}

#[test]
fn callback_can_only_be_called_by_root() {
    new_test_ext().execute_with(|| {
        init_chain();

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
                guild: [0; 32],
                role: [1; 32],
            },
        )
        .encode();

        let reqcheck_access = dummy_answer(
            vec![u8::from(true)],
            3,
            RequestData::ReqCheck {
                account: 1,
                guild: [0; 32],
                role: [1; 32],
            },
        )
        .encode();

        let test_data = vec![
            (
                <Guild>::callback(RuntimeOrigin::signed(1), vec![]),
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
        ];

        for (call, raw_error) in test_data {
            assert_eq!(error_msg(call.unwrap_err()), raw_error);
        }
    });
}
