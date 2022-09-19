#![cfg(test)]

use super::*;
#[test]
fn guild_interactions_work() {
    let mut ext: sp_io::TestExternalities = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into();
    ext.execute_with(|| {
        assert_ok!(Guild::create_guild(Origin::signed(4), 444));
        assert!(Guild::guilds(444).is_some());
        assert_ok!(Guild::join_guild(Origin::signed(4), 444));
        assert_eq!(Guild::guilds(444).unwrap().members().len(), 1);
        assert_eq!(Guild::guilds(444).unwrap().members()[0], 4);
        assert_noop!(
            Guild::create_guild(Origin::signed(4), 444),
            Error::<Test>::GuildAlreadyExists
        );
        assert_noop!(
            Guild::create_guild(Origin::signed(5), 444),
            Error::<Test>::GuildAlreadyExists
        );
        assert_noop!(
            Guild::join_guild(Origin::signed(4), 444),
            Error::<Test>::SignerAlreadyJoined
        );
        assert_ok!(Guild::join_guild(Origin::signed(5), 444));
        assert_ok!(Guild::join_guild(Origin::signed(6), 444));
        assert_ok!(Guild::join_guild(Origin::signed(7), 444));
        assert_ok!(Guild::join_guild(Origin::signed(8), 444));
        assert_eq!(Guild::guilds(444).unwrap().members().len(), 5);
        assert_noop!(
            Guild::join_guild(Origin::signed(7), 444),
            Error::<Test>::SignerAlreadyJoined
        );
        assert_noop!(
            Guild::join_guild(Origin::signed(8), 446),
            Error::<Test>::GuildDoesNotExist
        );
        assert_ok!(Guild::create_guild(Origin::signed(1), 446));
        assert_ok!(Guild::join_guild(Origin::signed(8), 446));
    });
}
