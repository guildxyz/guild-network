use crate::eth::EthSigner;
use futures::future::try_join_all;
use gn_api::query;
use gn_api::tx::{self, Signer, TxStatus};
use gn_api::{AccountId, Api};
use gn_common::filter::{Guild as GuildFilter, Logic as FilterLogic};
use gn_common::merkle::Proof as MerkleProof;
use gn_common::pad::{pad_to_n_bytes, unpad_from_n_bytes};
use gn_common::{Identity, Prefix};
use gn_test_data::*;

use std::sync::Arc;

pub fn dummy_users() -> Vec<Arc<EthSigner>> {
    let mut seed = ACCOUNT_SEED;
    (0..N_TEST_ACCOUNTS)
        .map(|_| {
            let signer = Arc::new(EthSigner::from_seed(seed));
            seed[0] += 1;
            signer
        })
        .inspect(|acc| println!("new evm account: {}", acc.account_id()))
        .collect()
}

pub async fn create_dummy_guilds(api: Api, signer: Arc<Signer>, accounts: &[Arc<EthSigner>]) {
    // create two guilds
    tx::send::ready(
        api.clone(),
        &tx::guild::create_guild(FIRST_GUILD, vec![1, 2, 3]),
        Arc::clone(&signer),
    )
    .await
    .expect("failed to create guild");

    println!("first guild created");

    tx::send::in_block(
        api.clone(),
        &tx::guild::create_guild(SECOND_GUILD, vec![4, 5, 6]),
        Arc::clone(&signer),
    )
    .await
    .expect("failed to create guild");

    println!("second guild created");

    let allowlist: Vec<AccountId> = accounts
        .iter()
        .map(|acc| acc.as_ref().account_id().clone())
        .collect();

    let filter = GuildFilter {
        name: FIRST_GUILD,
        role: Some(FIRST_ROLE),
    };
    // add one free and one filtered role to each guild
    // NOTE cannot try-join them because of different `impl TxPayload` opaque types
    tx::send::ready(
        api.clone(),
        &tx::guild::create_free_role(FIRST_GUILD, FIRST_ROLE),
        Arc::clone(&signer),
    )
    .await
    .unwrap();
    tx::send::ready(
        api.clone(),
        &tx::guild::create_free_role(SECOND_GUILD, FIRST_ROLE),
        Arc::clone(&signer),
    )
    .await
    .unwrap();
    tx::send::ready(
        api.clone(),
        &tx::guild::create_role_with_allowlist(
            FIRST_GUILD,
            SECOND_ROLE,
            allowlist,
            FilterLogic::And,
            None,
        )
        .unwrap(),
        Arc::clone(&signer),
    )
    .await
    .unwrap();
    tx::send::in_block(
        api.clone(),
        &tx::guild::create_child_role(SECOND_GUILD, SECOND_ROLE, filter, FilterLogic::Or, None)
            .unwrap(),
        signer,
    )
    .await
    .unwrap();

    println!("all roles created");
}

pub async fn join_guilds(api: Api, users: &[Arc<EthSigner>]) {
    // everybody joins the first guild's free role
    let payload = tx::guild::join_free_role(FIRST_GUILD, FIRST_ROLE);
    let join_request_futures = users
        .iter()
        .map(|acc| tx::send::in_block(api.clone(), &payload, Arc::clone(acc)))
        .collect::<Vec<_>>();

    try_join_all(join_request_futures).await.unwrap();

    println!("first guild first role joined");

    // only 2 joins the allowlist
    let allowlist = query::guild::allowlist(api.clone(), FIRST_GUILD, SECOND_ROLE)
        .await
        .unwrap();

    let proof_0 = MerkleProof::new(&allowlist, 0);
    let proof_1 = MerkleProof::new(&allowlist, 1);

    let payloads = vec![
        tx::guild::join_role_with_allowlist(FIRST_GUILD, SECOND_ROLE, proof_0),
        tx::guild::join_role_with_allowlist(FIRST_GUILD, SECOND_ROLE, proof_1),
    ];

    let join_request_futures = users
        .iter()
        .take(2)
        .enumerate()
        .map(|(i, acc)| tx::send::in_block(api.clone(), &payloads[i], Arc::clone(acc)))
        .collect::<Vec<_>>();

    try_join_all(join_request_futures).await.unwrap();

    println!("first guild second role joined");

    // only 5 joins the child role (they are all registered in first guild's
    // first role
    let payload = tx::guild::join_child_role(SECOND_GUILD, SECOND_ROLE);
    let join_request_futures = users
        .iter()
        .take(5)
        .map(|acc| tx::send::in_block(api.clone(), &payload, Arc::clone(acc)))
        .collect::<Vec<_>>();

    try_join_all(join_request_futures).await.unwrap();

    println!("second guild second role joined");

    // other 5 joins the free role of the second guild
    let payload = tx::guild::join_free_role(SECOND_GUILD, FIRST_ROLE);
    let join_request_futures = users
        .iter()
        .skip(5)
        .map(|acc| tx::send::in_block(api.clone(), &payload, Arc::clone(acc)))
        .collect::<Vec<_>>();

    try_join_all(join_request_futures).await.unwrap();

    println!("second guild first role joined");
}

pub async fn register_users(api: Api, users: &[Arc<EthSigner>]) {
    let register_address_payloads = users
        .iter()
        .map(|_| tx::identity::register())
        .collect::<Vec<_>>();

    let register_discord_payloads = users
        .iter()
        .enumerate()
        .map(|(i, _)| {
            tx::identity::link_identity(pad_to_n_bytes::<8, _>(b"discord"), [i as u8; 32])
        })
        .collect::<Vec<_>>();

    let register_futures = register_address_payloads
        .into_iter()
        .zip(users)
        .map(|(tx_payload, acc)| {
            tx::send::owned(api.clone(), tx_payload, Arc::clone(acc), TxStatus::InBlock)
        })
        .collect::<Vec<_>>();

    try_join_all(register_futures)
        .await
        .expect("failed to register accounts");

    println!("address registrations successfully submitted");

    let register_futures = register_discord_payloads
        .into_iter()
        .zip(users)
        .map(|(tx_payload, acc)| {
            tx::send::owned(api.clone(), tx_payload, Arc::clone(acc), TxStatus::InBlock)
        })
        .collect::<Vec<_>>();

    try_join_all(register_futures)
        .await
        .expect("failed to register discord");

    println!("discord registrations successfully submitted");
}

pub async fn wait_for_members(api: Api, filter: &GuildFilter, member_count: usize) {
    let mut i = 1;
    loop {
        let members = query::guild::members(api.clone(), filter, PAGE_SIZE)
            .await
            .expect("failed to query members");
        if members.len() == member_count {
            if let Some(role) = filter.role {
                println!(
                    "found {member_count} member(s) in role \"{}\" of guild \"{}\":",
                    unpad_from_n_bytes::<32>(&role),
                    unpad_from_n_bytes::<32>(&filter.name)
                );
            } else {
                println!(
                    "found {member_count} member(s) in guild \"{}\":",
                    unpad_from_n_bytes::<32>(&filter.name)
                );
            }
            members.iter().for_each(|member| println!("\t{member}"));
            break;
        } else {
            println!("waiting for members... (retries: {i}/{RETRIES})");
            if i == RETRIES {
                panic!("found {} members, expected {member_count}", members.len());
            }
            i += 1;
            tokio::time::sleep(std::time::Duration::from_millis(SLEEP_DURATION_MS)).await;
        }
    }
}

pub async fn wait_for_identity(
    api: Api,
    account: &AccountId,
    prefix: &Prefix,
    identity: &Identity,
) {
    let mut i = 1;
    loop {
        let identity_map = query::identity::identities(api.clone(), account)
            .await
            .expect("failed to fetch user identities");
        if let Some(id) = identity_map.get(prefix) {
            assert_eq!(id, identity);
            println!("USER ID REGISTERED");
            break;
        } else {
            println!("waiting for id registration... (retries: {i}/{RETRIES})");
            if i == RETRIES {
                panic!("couldn't find identity with prefix {prefix:?}")
            }
            i += 1;
            tokio::time::sleep(std::time::Duration::from_millis(SLEEP_DURATION_MS)).await;
        }
    }
}
