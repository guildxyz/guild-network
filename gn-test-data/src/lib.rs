pub const ACCOUNT_SEED: [u8; 32] = [10; 32];

// myrole
pub const FIRST_ROLE: [u8; 32] = [
    109, 121, 114, 111, 108, 101, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0,
];
// mysecondrole
pub const SECOND_ROLE: [u8; 32] = [
    109, 121, 115, 101, 99, 111, 110, 100, 114, 111, 108, 101, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
];
// myguild
pub const FIRST_GUILD: [u8; 32] = [
    109, 121, 103, 117, 105, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0,
];
// mysecondguild
pub const SECOND_GUILD: [u8; 32] = [
    109, 121, 115, 101, 99, 111, 110, 100, 103, 117, 105, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub const N_TEST_ACCOUNTS: usize = 10;
pub const PAGE_SIZE: u32 = 10;
pub const URL: &str = "ws://127.0.0.1:9944";