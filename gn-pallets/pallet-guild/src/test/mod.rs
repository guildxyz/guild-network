mod helpers;
//mod register;
//mod role;

use helpers::*;

use crate::mock::*;
type AccountId = <TestRuntime as frame_system::Config>::AccountId;

use frame_support::traits::{OnFinalize, OnInitialize};
use gn_common::{
    identities::{Identity, IdentityWithAuth},
    Request, RequestData,
};
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

        assert_eq!(
            last_event(),
            Event::Guild(pallet_guild::Event::GuildCreated(signer, guild_name))
        );

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
fn callback_can_only_be_called_by_root() {
    new_test_ext().execute_with(|| {
        init_chain();
        let error = <Guild>::callback(Origin::signed(1), vec![]).unwrap_err();
        assert_eq!(error, DispatchError::BadOrigin,);

        let error = <Guild>::callback(Origin::root(), vec![1]).unwrap_err();
        assert_eq!(error_msg(error), "CodecError");

        let no_access = dummy_answer(vec![u8::from(false)], 1, RequestData::Register(vec![]));
        let error = <Guild>::callback(Origin::root(), no_access.encode()).unwrap_err();
        assert_eq!(error_msg(error), "AccessDenied");

        let access = dummy_answer(
            vec![u8::from(true)],
            2,
            RequestData::ReqCheck {
                account: 1,
                guild: [0; 32],
                role: [1; 32],
            },
        );
        let error = <Guild>::callback(Origin::root(), access.encode()).unwrap_err();
        assert_eq!(error_msg(error), "GuildDoesNotExist");
    });
}
