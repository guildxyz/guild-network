use ethers::signers::{LocalWallet, Signer as EthSigner};
use futures::future::try_join_all;
use gn_client::query;
use gn_client::tx::{self, Keypair, PairT, Signer, TxStatus};
use gn_client::{AccountId, Api};
use gn_common::filter::{Guild as GuildFilter, Logic as FilterLogic};
use gn_common::identity::{EcdsaSignature, Identity, IdentityWithAuth};
use gn_common::merkle::Proof as MerkleProof;
use gn_test_data::*;
use rand::{rngs::StdRng, SeedableRng};
use sp_keyring::AccountKeyring;

use std::collections::BTreeMap;
use std::sync::Arc;

const RETRIES: u8 = 10;
const SLEEP_DURATION_MS: u64 = 1000;

pub struct Accounts {
    pub substrate: Arc<Signer>,
    pub eth: LocalWallet,
}

pub async fn api_with_alice(url: String) -> (Api, Arc<Signer>) {
    let api = Api::from_url(url)
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
    let mut seed = ACCOUNT_SEED;
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
        .inspect(|(id, _)| println!("new account: {id}"))
        .collect::<BTreeMap<AccountId, Accounts>>();

    let amount = 1_000_000_000u128;
    let mut keys = accounts.keys();
    // skip first
    let skipped_account = keys.next().unwrap();
    for account in keys {
        let tx = tx::fund_account(account, amount);
        tx::send_tx(api.clone(), &tx, Arc::clone(&faucet), TxStatus::Ready)
            .await
            .expect("failed to fund account");
    }
    // wait for the skipped one to be included in a block
    let tx = tx::fund_account(skipped_account, amount);
    tx::send_tx(api.clone(), &tx, faucet, TxStatus::InBlock)
        .await
        .expect("failed to fund account");

    println!("balance transfers in block");

    accounts
}

#[cfg(not(feature = "external-oracle"))]
pub async fn register_operators(
    api: Api,
    root: Arc<Signer>,
    accounts: impl Iterator<Item = &Accounts>,
) {
    println!("registring operators");
    for (i, account) in accounts.enumerate() {
        let payload = tx::register_operator(account.substrate.account_id());
        tx::send_tx_in_block(api.clone(), &tx::sudo(payload), Arc::clone(&root))
            .await
            .unwrap();
        println!("\toperator {i} registered");
    }

    println!("operator registrations in block");
}

#[cfg(not(feature = "external-oracle"))]
pub async fn activate_operators(api: Api, accounts: impl Iterator<Item = &Accounts>) {
    println!("activating operators");
    let tx_futures = accounts
        .map(|acc| {
            tx::send_owned_tx(
                api.clone(),
                tx::activate_operator(),
                Arc::clone(&acc.substrate),
                TxStatus::InBlock,
            )
        })
        .collect::<Vec<_>>();

    try_join_all(tx_futures).await.unwrap();

    println!("operators activated");
}

pub async fn wait_for_active_operator(api: Api) {
    let mut i = 0;
    loop {
        let active_operators = query::active_operators(api.clone())
            .await
            .expect("failed to fetch active operators");
        if active_operators.is_empty() {
            i += 1;
            println!("waiting for active operators");
            if i == RETRIES {
                panic!("no active operators found");
            }
            tokio::time::sleep(std::time::Duration::from_millis(SLEEP_DURATION_MS)).await;
        } else {
            println!("found an active operator");
            break;
        }
    }
}

pub async fn wait_for_oracle_answers(api: Api) {
    let mut i = 0;
    loop {
        let oracle_requests = query::oracle_requests(api.clone(), PAGE_SIZE)
            .await
            .expect("failed to fetch oracle requests");
        if !oracle_requests.is_empty() {
            i += 1;
            if i == RETRIES {
                panic!("ran out of retries while checking oracle requests")
            }
            tokio::time::sleep(std::time::Duration::from_millis(SLEEP_DURATION_MS)).await;
        } else {
            break;
        }
    }
}

pub async fn create_dummy_guilds(
    api: Api,
    signer: Arc<Signer>,
    accounts: impl Iterator<Item = &Accounts>,
) {
    // create two guilds
    tx::send_tx_ready(
        api.clone(),
        &tx::create_guild(FIRST_GUILD, vec![1, 2, 3]),
        Arc::clone(&signer),
    )
    .await
    .expect("failed to create guild");

    println!("first guild created");

    tx::send_tx_in_block(
        api.clone(),
        &tx::create_guild(SECOND_GUILD, vec![4, 5, 6]),
        Arc::clone(&signer),
    )
    .await
    .expect("failed to create guild");

    println!("second guild created");

    let allowlist: Vec<Identity> = accounts
        .map(|acc| Identity::Address20(acc.eth.address().to_fixed_bytes()))
        .collect();

    let filter = GuildFilter {
        name: FIRST_GUILD,
        role: Some(FIRST_ROLE),
    };
    // add one free and one filtered role to each guild
    // NOTE cannot try-join them because of different `impl TxPayload` opaque types
    tx::send_tx_ready(
        api.clone(),
        &tx::create_free_role(FIRST_GUILD, FIRST_ROLE),
        Arc::clone(&signer),
    )
    .await
    .unwrap();
    tx::send_tx_ready(
        api.clone(),
        &tx::create_free_role(SECOND_GUILD, FIRST_ROLE),
        Arc::clone(&signer),
    )
    .await
    .unwrap();
    tx::send_tx_ready(
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
    tx::send_tx_in_block(
        api.clone(),
        &tx::create_child_role(SECOND_GUILD, SECOND_ROLE, filter, FilterLogic::Or, None).unwrap(),
        signer,
    )
    .await
    .unwrap();

    println!("all roles created");
}

pub async fn join_guilds(api: Api, users: &BTreeMap<AccountId, Accounts>) {
    // everybody joins the first guild's free role
    let payload = tx::join(FIRST_GUILD, FIRST_ROLE, None);
    let join_request_futures = users
        .iter()
        .map(|(_, accounts)| {
            tx::send_tx_in_block(api.clone(), &payload, Arc::clone(&accounts.substrate))
        })
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
        .map(|(i, (_, accounts))| {
            tx::send_tx_in_block(api.clone(), &payloads[i], Arc::clone(&accounts.substrate))
        })
        .collect::<Vec<_>>();

    try_join_all(join_request_futures).await.unwrap();

    println!("first guild second role joined");

    // only 5 joins the child role (they are all registered in first guild's
    // first role
    let payload = tx::join(SECOND_GUILD, SECOND_ROLE, None);
    let join_request_futures = users
        .iter()
        .take(5)
        .map(|(_, accounts)| {
            tx::send_tx_in_block(api.clone(), &payload, Arc::clone(&accounts.substrate))
        })
        .collect::<Vec<_>>();

    try_join_all(join_request_futures).await.unwrap();

    println!("second guild second role joined");

    // other 5 joins the free role of the second guild
    let payload = tx::join(SECOND_GUILD, FIRST_ROLE, None);
    let join_request_futures = users
        .iter()
        .skip(5)
        .map(|(_, accounts)| {
            tx::send_tx_in_block(api.clone(), &payload, Arc::clone(&accounts.substrate))
        })
        .collect::<Vec<_>>();

    try_join_all(join_request_futures).await.unwrap();

    println!("second guild first role joined");
}

pub async fn register_users(api: Api, users: &BTreeMap<AccountId, Accounts>) {
    let signature_futures = users
        .iter()
        .map(|(id, accounts)| {
            let msg = gn_common::utils::verification_msg(id);

            accounts.eth.sign_message(msg)
        })
        .collect::<Vec<_>>();

    let signatures = try_join_all(signature_futures)
        .await
        .expect("failed to sign messages");

    let register_address_payloads = signatures
        .into_iter()
        .zip(users.iter())
        .map(|(sig, (_, accounts))| {
            let mut sig: [u8; 65] = sig.to_vec().try_into().unwrap();
            sig[64] -= 27; // due to eip-155 stuff in ethers
            let id_with_auth = IdentityWithAuth::Ecdsa(
                Identity::Address20(accounts.eth.address().to_fixed_bytes()),
                EcdsaSignature(sig),
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
        .zip(users.iter())
        .map(|(tx_payload, (_, accounts))| {
            tx::send_owned_tx(
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

    println!("address registrations successfully submitted");

    let register_futures = register_discord_payloads
        .into_iter()
        .zip(users.iter())
        .map(|(tx_payload, (_, accounts))| {
            tx::send_owned_tx(
                api.clone(),
                tx_payload,
                Arc::clone(&accounts.substrate),
                TxStatus::InBlock,
            )
        })
        .collect::<Vec<_>>();

    try_join_all(register_futures)
        .await
        .expect("failed to register discord");

    println!("discord registrations successfully submitted");
}

#[cfg(not(feature = "external-oracle"))]
pub async fn send_dummy_oracle_answers(api: Api, operators: &BTreeMap<AccountId, Accounts>) {
    let oracle_requests = query::oracle_requests(api.clone(), PAGE_SIZE)
        .await
        .expect("failed to fetch oracle requests");

    let oracle_answer_futures = oracle_requests
        .into_iter()
        .map(|(request_id, operator)| {
            let tx = tx::oracle_callback(request_id, vec![u8::from(true)]);
            let accounts = operators.get(&operator).unwrap();
            tx::send_owned_tx(
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

    println!("oracle requests successfully answered");
}
