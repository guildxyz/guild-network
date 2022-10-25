use crate as pallet_guild;
use test_runtime::test_runtime;

use sp_runtime::DispatchError;

test_runtime!(Guild, pallet_guild);

fn error_msg<'a>(error: DispatchError) -> &'a str {
    match error {
        DispatchError::Module(module_error) => module_error.message.unwrap(),
        _ => panic!("unexpected error"),
    }
}

// TODO add more tests once guild functionalities are final
#[test]
fn create_guild() {
    new_test_runtime().execute_with(|| {
        System::set_block_number(1);
        assert!(<Guild>::create_guild(Origin::signed(4), 444, 1000).is_ok());
    });
}

#[test]
fn callback_can_only_be_called_by_root() {
    new_test_runtime().execute_with(|| {
        System::set_block_number(1);
        let error = <Guild>::callback(Origin::signed(1), vec![]).err().unwrap();
        assert_eq!(error, DispatchError::BadOrigin,);

        let error = <Guild>::callback(Origin::root(), vec![]).err().unwrap();
        assert_eq!(error_msg(error), "InvalidResultLength");
    });
}
