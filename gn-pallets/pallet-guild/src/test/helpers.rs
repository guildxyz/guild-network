use super::*;
use gn_common::GuildName;

pub const METADATA: &[u8] =
    &[12u8; <TestRuntime as pallet_guild::Config>::MaxSerializedLen::get() as usize];

pub type GuildEvent = pallet_guild::Event<TestRuntime>;
pub type GuildError = pallet_guild::Error<TestRuntime>;
pub type AccountId = <TestRuntime as frame_system::Config>::AccountId;

pub fn last_event() -> pallet_guild::Event<TestRuntime> {
    System::events()
        .into_iter()
        .filter_map(|e| {
            if let RuntimeEvent::Guild(inner) = e.event {
                Some(inner)
            } else {
                None
            }
        })
        .last()
        .unwrap()
}

pub fn dummy_guild(signer: AccountId, guild_name: GuildName) {
    <Guild>::create_guild(RuntimeOrigin::signed(signer), guild_name, METADATA.to_vec()).unwrap();
    assert_eq!(last_event(), GuildEvent::GuildCreated(signer, guild_name));
    let guild_id = <Guild>::guild_id(guild_name).unwrap();
    let guild = <Guild>::guild(guild_id).unwrap();
    assert_eq!(guild.name, guild_name);
    assert_eq!(guild.owner, signer);
    assert_eq!(guild.metadata, METADATA);
    assert!(guild.roles.is_empty());
}
