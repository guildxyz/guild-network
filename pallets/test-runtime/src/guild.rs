use super::*;

#[test]
fn create_guild() {
    new_test_runtime().execute_with(|| {
        System::set_block_number(1);
        assert!(<Guild>::create_guild(Origin::signed(4), 444, 1000).is_ok());
    });
}
