use super::*;

const STARTING_BLOCK_NUM: u64 = 2;

pub const METADATA: &[u8] =
    &[12u8; <TestRuntime as pallet_guild::Config>::MaxSerializedLen::get() as usize];

pub fn new_test_ext() -> sp_io::TestExternalities {
    frame_system::GenesisConfig::default()
        .build_storage::<TestRuntime>()
        .unwrap()
        .into()
}

pub fn init_chain() {
    for i in 0..STARTING_BLOCK_NUM {
        System::set_block_number(i);
        <RandomnessCollectiveFlip as OnInitialize<u64>>::on_initialize(i);
        <RandomnessCollectiveFlip as OnFinalize<u64>>::on_finalize(i);
    }
}

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

pub fn error_msg<'a>(error: DispatchError) -> &'a str {
    match error {
        DispatchError::Module(module_error) => module_error.message.unwrap(),
        DispatchError::BadOrigin => "BadOrigin",
        _ => panic!("unexpected error"),
    }
}

pub fn dummy_answer(
    result: Vec<u8>,
    requester: AccountId,
    request_data: RequestData<AccountId>,
) -> pallet_oracle::OracleAnswer {
    let data = gn_common::Request::<AccountId> {
        requester,
        data: request_data,
    }
    .encode();
    pallet_oracle::OracleAnswer { data, result }
}

// successfully create a guild
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
