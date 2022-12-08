use futures::future::try_join_all;
use gn_client::data::*;
#[cfg(not(feature = "external-oracle"))]
use gn_client::queries::*;
use gn_client::transactions::*;
use gn_client::{AccountId, Api, Keypair, Signer, TxStatus};
use gn_common::{GuildName, RoleName};
use gn_gate::requirements::Requirement;
use sp_keyring::AccountKeyring;
use subxt::ext::sp_core::crypto::Pair as TraitPair;

use std::collections::BTreeMap;
use std::sync::Arc;

const URL: &str = "ws://127.0.0.1:9944";
pub const FIRST_ROLE: RoleName = [0; 32];
pub const SECOND_ROLE: RoleName = [1; 32];
pub const FIRST_GUILD: GuildName = [2; 32];
pub const SECOND_GUILD: GuildName = [3; 32];
pub const PAGE_SIZE: u32 = 10;

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
) -> BTreeMap<AccountId, Arc<Signer>> {
    let mut seed = [10u8; 32];
    let accounts = (0..num_accounts)
        .map(|_| {
            let keypair = Arc::new(Signer::new(Keypair::from_seed(&seed)));
            seed[0] += 1;
            (keypair.as_ref().account_id().clone(), keypair)
        })
        .inspect(|(id, _)| println!("new account: {}", id))
        .collect::<BTreeMap<AccountId, Arc<Signer>>>();

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

pub async fn register_operators(api: Api, operators: impl Iterator<Item = &Arc<Signer>>) {
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

    println!("operator registrations in block");
}

pub async fn create_dummy_guilds(api: Api, signer: Arc<Signer>) {
    // create two guilds, each with 2 roles
    let roles = vec![
        Role {
            name: FIRST_ROLE,
            requirements: RequirementsLogic {
                logic: "".to_string(),
                requirements: vec![Requirement::Free],
            },
        },
        Role {
            name: SECOND_ROLE,
            requirements: RequirementsLogic {
                logic: "".to_string(),
                requirements: vec![Requirement::Free],
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

pub async fn join_guilds(api: Api, operators: &BTreeMap<AccountId, Arc<Signer>>) {
    let join_request_txns = [
        join_guild(FIRST_GUILD, FIRST_ROLE, vec![], vec![]).expect("Failed to serialize data"),
        join_guild(FIRST_GUILD, SECOND_ROLE, vec![], vec![]).expect("Failed to serialize data"),
        join_guild(SECOND_GUILD, FIRST_ROLE, vec![], vec![]).expect("Failed to serialize data"),
        join_guild(SECOND_GUILD, SECOND_ROLE, vec![], vec![]).expect("Failed to serialize data"),
    ];

    let join_request_futures = operators
        .values()
        .enumerate()
        .map(|(i, signer)| {
            send_tx_in_block(api.clone(), &join_request_txns[i % 4], Arc::clone(signer))
        })
        .collect::<Vec<_>>();

    try_join_all(join_request_futures)
        .await
        .expect("failed to submit oracle answers");

    println!("join requests successfully submitted");
}

#[cfg(not(feature = "external-oracle"))]
pub async fn send_dummy_oracle_answers(api: Api, operators: &BTreeMap<AccountId, Arc<Signer>>) {
    let oracle_requests = oracle_requests(api.clone(), PAGE_SIZE)
        .await
        .expect("failed to fetch oracle requests");

    let oracle_answer_futures = oracle_requests
        .into_iter()
        .map(|(request_id, operator)| {
            let tx = oracle_callback(request_id, vec![u8::from(true)]);
            let signer = operators.get(&operator).unwrap();
            send_owned_tx(api.clone(), tx, Arc::clone(signer), TxStatus::InBlock)
        })
        .collect::<Vec<_>>();

    try_join_all(oracle_answer_futures)
        .await
        .expect("failed to submit oracle answers");

    println!("join requests successfully answered");
}
