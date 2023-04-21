use crate::eth::EthSigner;
use ethers::types::{Address, U256};
use futures::future::try_join_all;
use gn_api::query;
use gn_api::tx::{self, Signer, SignerT, TxStatus};
use gn_api::{AccountId, Api};
use gn_common::filter::{Guild as GuildFilter, Logic as FilterLogic};
use gn_common::identity::{EcdsaSignature, Identity, IdentityWithAuth};
use gn_common::merkle::Proof as MerkleProof;
use gn_common::pad::unpad_from_n_bytes;
use gn_engine::balance::{Balance, Relation, TokenType};
use gn_engine::chains::EvmChain;
use gn_engine::{Requirement, RequirementsWithLogic};
use gn_test_data::*;

use std::str::FromStr;
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
        &tx::create_guild(FIRST_GUILD, vec![1, 2, 3]),
        Arc::clone(&signer),
    )
    .await
    .expect("failed to create guild");

    println!("first guild created");

    tx::send::in_block(
        api.clone(),
        &tx::create_guild(SECOND_GUILD, vec![4, 5, 6]),
        Arc::clone(&signer),
    )
    .await
    .expect("failed to create guild");

    println!("second guild created");

    let allowlist: Vec<Identity> = accounts
        .iter()
        .map(|acc| Identity::Address20(acc.as_ref().evm_address()))
        .collect();

    let filter = GuildFilter {
        name: FIRST_GUILD,
        role: Some(FIRST_ROLE),
    };
    // add one free and one filtered role to each guild
    // NOTE cannot try-join them because of different `impl TxPayload` opaque types
    tx::send::ready(
        api.clone(),
        &tx::create_free_role(FIRST_GUILD, FIRST_ROLE),
        Arc::clone(&signer),
    )
    .await
    .unwrap();
    tx::send::ready(
        api.clone(),
        &tx::create_free_role(SECOND_GUILD, FIRST_ROLE),
        Arc::clone(&signer),
    )
    .await
    .unwrap();
    tx::send::ready(
        api.clone(),
        &tx::create_role_with_allowlist(
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
        &tx::create_child_role(SECOND_GUILD, SECOND_ROLE, filter, FilterLogic::Or, None).unwrap(),
        signer,
    )
    .await
    .unwrap();

    println!("all roles created");
}

pub async fn join_guilds(api: Api, users: &[Arc<EthSigner>]) {
    // everybody joins the first guild's free role
    let payload = tx::join(FIRST_GUILD, FIRST_ROLE, None);
    let join_request_futures = users
        .iter()
        .map(|acc| tx::send::in_block(api.clone(), &payload, Arc::clone(acc)))
        .collect::<Vec<_>>();

    try_join_all(join_request_futures).await.unwrap();

    println!("first guild first role joined");

    // only 2 joins the allowlist
    let allowlist = query::allowlist(api.clone(), FIRST_GUILD, SECOND_ROLE)
        .await
        .unwrap()
        .unwrap();

    let proof_0 = MerkleProof::new(&allowlist, 0, 0);
    let proof_1 = MerkleProof::new(&allowlist, 1, 0);

    let payloads = vec![
        tx::join(FIRST_GUILD, SECOND_ROLE, Some(proof_0)),
        tx::join(FIRST_GUILD, SECOND_ROLE, Some(proof_1)),
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
    let payload = tx::join(SECOND_GUILD, SECOND_ROLE, None);
    let join_request_futures = users
        .iter()
        .take(5)
        .map(|acc| tx::send::in_block(api.clone(), &payload, Arc::clone(acc)))
        .collect::<Vec<_>>();

    try_join_all(join_request_futures).await.unwrap();

    println!("second guild second role joined");

    // other 5 joins the free role of the second guild
    let payload = tx::join(SECOND_GUILD, FIRST_ROLE, None);
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
        .map(|acc| {
            let msg = gn_common::utils::verification_msg(acc.account_id());

            let signature = match acc.sign(msg.as_bytes()) {
                subxt::utils::MultiSignature::Ecdsa(sig) => sig,
                _ => unreachable!(),
            };

            let id_with_auth = IdentityWithAuth::Ecdsa(
                Identity::Address20(acc.evm_address()),
                EcdsaSignature(signature),
            );
            tx::register(id_with_auth, 0)
        })
        .collect::<Vec<_>>();

    let register_discord_payloads = users
        .iter()
        .enumerate()
        .map(|(i, _)| {
            tx::register(
                IdentityWithAuth::Other(
                    Identity::Other(gn_common::pad::padded_id(b"discord:", i as u64)),
                    [0u8; 64],
                ),
                1,
            )
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
    let mut i = 0;
    loop {
        let members = query::members(api.clone(), filter, PAGE_SIZE)
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
            i += 1;
            if i == RETRIES {
                panic!("found {} members, expected {member_count}", members.len());
            }
            tokio::time::sleep(std::time::Duration::from_millis(SLEEP_DURATION_MS)).await;
        }
    }
}

pub async fn wait_for_identity(api: Api, account: &AccountId, identity: &Identity, index: usize) {
    let mut i = 0;
    loop {
        let user_identity = query::user_identity(api.clone(), account)
            .await
            .expect("failed to fetch user identities");
        if user_identity.len() > index {
            assert_eq!(user_identity.get(index), Some(identity));
            println!("USER ID REGISTERED");
            break;
        } else {
            i += 1;
            if i == RETRIES {
                panic!("couldn't find identity at index {index}")
            }
            tokio::time::sleep(std::time::Duration::from_millis(SLEEP_DURATION_MS)).await;
        }
    }
}

pub fn dummy_requirements_with_logic() -> (RequirementsWithLogic, RequirementsWithLogic) {
    let mut one = [0u8; 32];
    one[0] = 1;

    let first_reqs = RequirementsWithLogic {
        logic: "0 AND 1".to_string(),
        requirements: vec![
            Requirement::EvmBalance(Balance {
                token_type: Some(TokenType::NonFungible {
                    address: Address::from_str(ETH_ERC721_ADDRESS)
                        .unwrap()
                        .to_fixed_bytes(),
                    id: Some(U256::from_dec_str(ETH_ERC721_ID).unwrap().into()),
                }),
                relation: Relation::EqualTo(one),
                chain: EvmChain::Ethereum,
            }),
            Requirement::EvmBalance(Balance {
                token_type: None,
                relation: Relation::GreaterThan([0u8; 32]),
                chain: EvmChain::Ethereum,
            }),
        ],
    };
    let second_reqs = RequirementsWithLogic {
        logic: "0 OR (1 AND 2)".to_string(),
        requirements: vec![
            Requirement::EvmBalance(Balance {
                token_type: None,
                relation: Relation::GreaterThan([1u8; 32]),
                chain: EvmChain::Ethereum,
            }),
            Requirement::EvmBalance(Balance {
                token_type: Some(TokenType::NonFungible {
                    address: Address::from_str(GNOSIS_ERC721_ADDRESS_0)
                        .unwrap()
                        .to_fixed_bytes(),
                    id: Some(U256::from_dec_str(GNOSIS_ERC721_ID_0).unwrap().into()),
                }),
                relation: Relation::EqualTo(one),
                chain: EvmChain::Gnosis,
            }),
            Requirement::EvmBalance(Balance {
                token_type: Some(TokenType::NonFungible {
                    address: Address::from_str(GNOSIS_ERC721_ADDRESS_1)
                        .unwrap()
                        .to_fixed_bytes(),
                    id: Some(U256::from_dec_str(GNOSIS_ERC721_ID_1).unwrap().into()),
                }),
                relation: Relation::EqualTo(U256::from(1).into()),
                chain: EvmChain::Gnosis,
            }),
        ],
    };

    (first_reqs, second_reqs)
}
