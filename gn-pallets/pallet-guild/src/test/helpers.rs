use super::*;

const STARTING_BLOCK_NUM: u64 = 2;
pub const METADATA: &[u8] = &[9, 9, 9, 9, 0, 0, 0, 0, 0];

pub const ROLE_1: [u8; 32] = [1u8; 32];
pub const ROLE_2: [u8; 32] = [2u8; 32];
pub const ROLE_3: [u8; 32] = [3u8; 32];

pub const REQ_1: &[u8] = &[6, 7, 8, 9, 0];
pub const REQ_2: &[u8] = &[2, 4, 6, 8];
pub const REQ_3: &[u8] = &[1, 3, 5];

pub const LOGIC_1: &[u8] = &[0];
pub const LOGIC_2: &[u8] = &[1];
pub const LOGIC_3: &[u8] = &[2];

pub fn init_chain() {
    for i in 0..STARTING_BLOCK_NUM {
        System::set_block_number(i);
        <RandomnessCollectiveFlip as OnInitialize<u64>>::on_initialize(i);
        <RandomnessCollectiveFlip as OnFinalize<u64>>::on_finalize(i);
    }
}

pub fn last_event() -> Event {
    System::events()
        .into_iter()
        .filter_map(|e| {
            if let Event::Guild(inner) = e.event {
                Some(Event::Guild(inner))
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

pub fn new_guild(signer: AccountId, guild_name: [u8; 32]) {
    let roles = vec![
        (
            ROLE_1,
            (
                LOGIC_1.to_vec(),
                vec![REQ_1.to_vec(), REQ_2.to_vec(), REQ_3.to_vec()],
            ),
        ),
        (
            ROLE_2,
            (
                LOGIC_2.to_vec(),
                vec![REQ_1.to_vec(), REQ_2.to_vec(), REQ_3.to_vec()],
            ),
        ),
        (
            ROLE_3,
            (
                LOGIC_3.to_vec(),
                vec![REQ_1.to_vec(), REQ_2.to_vec(), REQ_3.to_vec()],
            ),
        ),
    ];

    assert!(
        <Guild>::create_guild(Origin::signed(signer), guild_name, METADATA.to_vec(), roles).is_ok()
    );
}
