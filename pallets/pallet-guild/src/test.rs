use crate as pallet_guild;
use test_runtime::test_runtime;

test_runtime!(Guild, pallet_guild);

// TODO add more tests once guild functionalities are final
#[test]
fn create_guild() {
    new_test_runtime().execute_with(|| {
        System::set_block_number(1);
        assert!(<Guild>::create_guild(Origin::signed(4), 444, 1000).is_ok());
    });
}
