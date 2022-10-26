use crate as pallet_guild;
use test_runtime::test_runtime;

use sp_runtime::DispatchError;

test_runtime!(Guild, pallet_guild);

pub fn last_event() -> Event {
    System::events()
        .into_iter()
        .map(|r| r.event)
        .filter_map(|e| {
            if let Event::Guild(inner) = e {
                Some(Event::Guild(inner))
            } else {
                None
            }
        })
        .last()
        .unwrap()
}

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
        let signer = 4;
        let guild_id = 444;
        assert!(<Guild>::create_guild(Origin::signed(signer), guild_id, 1000).is_ok());

        assert_eq!(
            last_event(),
            Event::Guild(pallet_guild::Event::GuildCreated(signer, guild_id))
        );
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

        let error = <Guild>::callback(
            Origin::root(),
            vec![255, 255, 255, 255, 255, 255, 255, 255, 1],
        )
        .err()
        .unwrap();
        assert_eq!(error_msg(error), "JoinRequestDoesNotExist");

        assert_eq!(
            last_event(),
            Event::Guild(pallet_guild::Event::DecodingComplete(u64::MAX, true))
        );

        let error = <Guild>::callback(Origin::root(), vec![0, 0, 0, 0, 0, 0, 0, 0, 0])
            .err()
            .unwrap();
        assert_eq!(error_msg(error), "JoinRequestDoesNotExist");
        assert_eq!(
            last_event(),
            Event::Guild(pallet_guild::Event::DecodingComplete(0, false))
        );
    });
}
