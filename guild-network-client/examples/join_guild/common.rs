use futures::future::try_join_all;
use guild_network_client::queries::*;
use guild_network_client::transactions::*;
use guild_network_client::{AccountId, Api, Guild, Keypair, Role, Signer, TxStatus};
use guild_network_gate::requirements::Requirement;
use sp_keyring::AccountKeyring;
use subxt::ext::sp_core::crypto::Pair as TraitPair;

use std::collections::{hash_map, HashMap};
use std::sync::Arc;

const URL: &str = "ws://127.0.0.1:9944";
pub const FIRST_ROLE: [u8; 32] = [0; 32];
pub const SECOND_ROLE: [u8; 32] = [1; 32];
pub const FIRST_GUILD: [u8; 32] = [2; 32];
pub const SECOND_GUILD: [u8; 32] = [3; 32];
pub const PAGE_SIZE: u32 = 10;

pub async fn api_with_alice() -> (Api, Arc<Signer>) {
    let api = Api::from_url(URL)
        .await
        .expect("failed to initialize client");
    let alice = Arc::new(Signer::new(AccountKeyring::Alice.pair()));

    (api, alice)
}

pub async fn prefunded_accounts(api: Api, faucet: Arc<Signer>) -> HashMap<AccountId, Arc<Signer>> {
    let mut seed = [10u8; 32];
    let num_accounts = 10;
    let accounts = (0..num_accounts)
        .map(|_| {
            let keypair = Arc::new(Signer::new(Keypair::from_seed(&seed)));
            seed[0] += 1;
            (keypair.as_ref().account_id().clone(), keypair)
        })
        .inspect(|(id, _)| println!("new account: {}", id))
        .collect::<HashMap<AccountId, Arc<Signer>>>();

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

    println!("Balance transfers in block!");

    accounts
}

pub async fn register_operators(api: Api, operators: hash_map::Values<'_, AccountId, Arc<Signer>>) {
    let register_operator_futures = operators
        .map(|operator| {
            send_owned_tx(
                api.clone(),
                register_operator(),
                Arc::clone(operator),
                TxStatus::InBlock,
            )
        })
        .collect::<Vec<_>>();

    try_join_all(register_operator_futures)
        .await
        .expect("failed to register operators");

    println!("Operator registrations in block!");
}

pub async fn create_dummy_guilds(api: Api, signer: Arc<Signer>) {
    // create two guilds, each with 2 roles
    let roles = vec![
        Role {
            name: FIRST_ROLE,
            requirements: vec![Requirement::Free],
        },
        Role {
            name: SECOND_ROLE,
            requirements: vec![Requirement::Free],
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

    send_tx_ready(api.clone(), &create_guild(first_guild), Arc::clone(&signer))
        .await
        .expect("failed to create guild");
    send_tx_in_block(api.clone(), &create_guild(second_guild), signer)
        .await
        .expect("failed to create guild");
}

pub async fn join_guilds(api: Api, mut operators: hash_map::Values<'_, AccountId, Arc<Signer>>) {
    let join_request_txns = [
        join_guild(FIRST_GUILD, FIRST_ROLE, vec![], vec![]),
        join_guild(FIRST_GUILD, SECOND_ROLE, vec![], vec![]),
        join_guild(SECOND_GUILD, FIRST_ROLE, vec![], vec![]),
        join_guild(SECOND_GUILD, SECOND_ROLE, vec![], vec![]),
    ];

    // next() returns an arbitrary signer
    let first_joiner = operators.next().unwrap();

    for (i, signer) in operators.enumerate() {
        send_tx_ready(api.clone(), &join_request_txns[i % 4], Arc::clone(&signer))
            .await
            .expect("failed to submit join request");
    }

    // first joiner will join all guilds
    for join_request_tx in join_request_txns.iter().skip(1) {
        send_tx_ready(api.clone(), join_request_tx, Arc::clone(&first_joiner))
            .await
            .expect("failed to submit join request");
    }

    send_tx_in_block(
        api.clone(),
        &join_request_txns[0],
        Arc::clone(&first_joiner),
    )
    .await
    .expect("failed to submit join request");
}

pub async fn send_dummy_oracle_answers(api: Api, operators: &HashMap<AccountId, Arc<Signer>>) {
    println!("ORACLE REQUESTS");
    let oracle_requests = oracle_requests(api.clone(), PAGE_SIZE)
        .await
        .expect("failed to fetch oracle requests");
    println!("{:#?}", oracle_requests);

    let oracle_answer_futures = oracle_requests
        .into_iter()
        .map(|(request_id, account_id)| {
            let tx = oracle_callback(request_id, vec![u8::from(true)]);
            let signer = operators.get(&account_id).unwrap();
            send_owned_tx(api.clone(), tx, Arc::clone(signer), TxStatus::InBlock)
        })
        .collect::<Vec<_>>();

    try_join_all(oracle_answer_futures)
        .await
        .expect("failed to submit oracle answers");

    println!("Join guild requests in block!");
}
