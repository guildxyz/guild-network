use ethers::signers::{LocalWallet, Signer as EthSigner};
use futures::future::try_join_all;
use guild_network_client::data::*;
#[cfg(not(feature = "external-oracle"))]
use guild_network_client::queries::*;
use guild_network_client::transactions::*;
use guild_network_client::{AccountId, Api, Hash, Keypair, RuntimeIdentityWithAuth, Signer};
use guild_network_common::requirements::{Requirement, RequirementsWithLogic};
use guild_network_common::{EvmAddress, EvmSignature, GuildName, RoleName};
use rand::{rngs::StdRng, SeedableRng};
use sp_keyring::AccountKeyring;
use subxt::ext::sp_core::crypto::Pair as TraitPair;

use std::collections::BTreeMap;
use std::sync::Arc;

const URL: &str = "ws://127.0.0.1:9944";
pub const FIRST_ROLE: RoleName = [0; 32];
pub const SECOND_ROLE: RoleName = [1; 32];
// myguild
pub const FIRST_GUILD: GuildName = [
    109, 121, 103, 117, 105, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0,
];
// mysecondguild
pub const SECOND_GUILD: GuildName = [
    109, 121, 115, 101, 99, 111, 110, 100, 103, 117, 105, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0,
];
pub const N_TEST_ACCOUNTS: usize = 10;
pub const PAGE_SIZE: u32 = 10;

pub struct Accounts {
    pub substrate: Arc<Signer>,
    pub eth: LocalWallet,
}

pub async fn api_with_alice() -> (Api, Arc<Signer>) {
    let api = Api::from_url(URL)
        .await
        .expect("failed to initialize client");
    let alice = Arc::new(Signer::new(AccountKeyring::Alice.pair()));

    (api, alice)
}

pub async fn prefunded_accounts(
    api: Api,
    faucet: Arc<Signer>,
    num_accounts: usize,
) -> BTreeMap<AccountId, Accounts> {
    let mut rng = StdRng::seed_from_u64(0);
    let mut seed = [10u8; 32];
    let accounts = (0..num_accounts)
        .map(|_| {
            let keypair = Arc::new(Signer::new(Keypair::from_seed(&seed)));
            seed[0] += 1;
            let accounts = Accounts {
                substrate: keypair,
                eth: LocalWallet::new(&mut rng),
            };
            (accounts.substrate.as_ref().account_id().clone(), accounts)
        })
        .inspect(|(id, _)| println!("new account: {:?}", id))
        .collect::<BTreeMap<AccountId, Accounts>>();

    let amount = 1_000_000_000_000_000u128;
    let mut keys = accounts.keys();
    // skip first
    let skipped_account = keys.next().unwrap();
    for account in keys {
        let tx = fund_account(account, amount);
        send_tx(api.clone(), &tx, Arc::clone(&faucet), TxStatus::Ready)
            .await
            .expect("failed to fund account");
    }
    // wait for the skipped one to be included in a block
    let tx = fund_account(skipped_account, amount);
    send_tx(api.clone(), &tx, faucet, TxStatus::InBlock)
        .await
        .expect("failed to fund account");

    println!("balance transfers in block");

    accounts
}

pub async fn register_operators(api: Api, accounts: impl Iterator<Item = &Accounts>) {
    let register_operator_futures = accounts
        .map(|account| {
            send_owned_tx(
                api.clone(),
                register_operator(),
                Arc::clone(&account.substrate),
                TxStatus::InBlock,
            )
        })
        .collect::<Vec<_>>();

    try_join_all(register_operator_futures)
        .await
        .expect("failed to register operators");

    println!("operator registrations in block");
}

pub async fn create_dummy_guilds(
    api: Api,
    signer: Arc<Signer>,
    accounts: impl Iterator<Item = &Accounts>,
) {
    let allowlist: Vec<EvmAddress> = accounts
        .map(|acc| acc.eth.address().to_fixed_bytes())
        .collect();
    // create two guilds, each with 2 roles
    let roles = vec![
        Role {
            name: FIRST_ROLE,
            reqs: RequirementsWithLogic {
                logic: "0".to_string(),
                requirements: vec![Requirement::Free],
            },
        },
        Role {
            name: SECOND_ROLE,
            reqs: RequirementsWithLogic {
                logic: "0".to_string(),
                requirements: vec![Requirement::EvmAllowlist(allowlist.into())],
            },
        },
    ];

    let first_guild = Guild {
        name: FIRST_GUILD,
        metadata: vec![1, 2, 3],
        roles: roles.clone(),
    };
    let second_guild = Guild {
        name: SECOND_GUILD,
        metadata: vec![4, 5, 6],
        roles,
    };

    send_tx_ready(
        api.clone(),
        &create_guild(first_guild).expect("Failed to serialize requirements"),
        Arc::clone(&signer),
    )
    .await
    .expect("failed to create guild");
    send_tx_in_block(
        api.clone(),
        &create_guild(second_guild).expect("Failed to serialize requirements"),
        signer,
    )
    .await
    .expect("failed to create guild");
}

pub async fn join_guilds(api: Api, users: &BTreeMap<AccountId, Accounts>) {
    let join_request_futures = users
        .iter()
        .enumerate()
        .map(|(i, (_, accounts))| match i % 4 {
            0 => join_request_tx(api.clone(), &FIRST_GUILD, &FIRST_ROLE, accounts),
            1 => join_request_tx(api.clone(), &FIRST_GUILD, &SECOND_ROLE, accounts),
            2 => join_request_tx(api.clone(), &SECOND_GUILD, &FIRST_ROLE, accounts),
            3 => join_request_tx(api.clone(), &SECOND_GUILD, &SECOND_ROLE, accounts),
            _ => unreachable!(),
        })
        .collect::<Vec<_>>();

    try_join_all(join_request_futures)
        .await
        .expect("failed to submit oracle answers");

    println!("join requests successfully submitted");
}

pub async fn register_users(api: Api, users: &BTreeMap<AccountId, Accounts>) {
    let signature_futures = users
        .iter()
        .map(|(id, accounts)| {
            let msg = guild_network_common::utils::verification_msg(id);

            accounts.eth.sign_message(msg)
        })
        .collect::<Vec<_>>();

    let signatures = try_join_all(signature_futures)
        .await
        .expect("failed to sign messages");

    let register_futures = signatures
        .into_iter()
        .zip(users.iter())
        .enumerate()
        .map(|(i, (sig, (_, accounts)))| {
            let tx_payload = register(vec![
                RuntimeIdentityWithAuth::EvmChain(
                    accounts.eth.address().to_fixed_bytes(),
                    // NOTE unwrap is fine because byte lengths always match
                    EvmSignature::try_from(sig.to_vec()).unwrap(),
                ),
                RuntimeIdentityWithAuth::Discord(i as u64, ()),
            ]);
            send_owned_tx(
                api.clone(),
                tx_payload,
                Arc::clone(&accounts.substrate),
                TxStatus::InBlock,
            )
        })
        .collect::<Vec<_>>();

    try_join_all(register_futures)
        .await
        .expect("failed to register accounts");

    println!("registrations successfully submitted");
}

async fn join_request_tx(
    api: Api,
    guild_name: &GuildName,
    role_name: &RoleName,
    accounts: &Accounts,
) -> Result<Hash, subxt::Error> {
    let tx_payload = join_guild(*guild_name, *role_name);
    send_tx_in_block(api, &tx_payload, Arc::clone(&accounts.substrate)).await
}

#[cfg(not(feature = "external-oracle"))]
pub async fn send_dummy_oracle_answers(api: Api, operators: &BTreeMap<AccountId, Accounts>) {
    let oracle_requests = oracle_requests(api.clone(), PAGE_SIZE)
        .await
        .expect("failed to fetch oracle requests");

    let oracle_answer_futures = oracle_requests
        .into_iter()
        .map(|(request_id, operator)| {
            let tx = oracle_callback(request_id, vec![u8::from(true)]);
            let accounts = operators.get(&operator).unwrap();
            send_owned_tx(
                api.clone(),
                tx,
                Arc::clone(&accounts.substrate),
                TxStatus::InBlock,
            )
        })
        .collect::<Vec<_>>();

    try_join_all(oracle_answer_futures)
        .await
        .expect("failed to submit oracle answers");

    println!("join requests successfully answered");
}
