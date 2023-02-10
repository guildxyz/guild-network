use ethers::signers::{LocalWallet, Signer as EthSigner};
use futures::future::try_join_all;
use gn_client::data::*;
#[cfg(not(feature = "external-oracle"))]
use gn_client::query;
use gn_client::runtime::runtime_types::sp_core::ecdsa::Signature as RuntimeEcdsaSignature;
use gn_client::tx::{self, Keypair, PairT, Signer, TxStatus};
use gn_client::{AccountId, Api, Hash, RuntimeIdentity, RuntimeIdentityWithAuth};
use gn_common::pad::pad_to_n_bytes;
use gn_common::requirements::{EvmAddress, Requirement, RequirementsWithLogic};
use gn_common::{GuildName, RoleName};
use gn_test_data::*;
use rand::{rngs::StdRng, SeedableRng};
use sp_keyring::AccountKeyring;

use std::collections::BTreeMap;
use std::sync::Arc;

pub fn discord_id(i: u64) -> [u8; 64] {
    let mut tmp = Vec::from(b"discord:".as_ref());
    tmp.extend_from_slice(i.to_le_bytes().as_ref());
    pad_to_n_bytes::<64, _>(tmp)
}

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

    let amount = 1_000_000_000_000_000u128;
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
pub async fn register_operators(api: Api, accounts: impl Iterator<Item = &Accounts>) {
    let register_operator_futures = accounts
        .map(|account| {
            tx::send_owned_tx(
                api.clone(),
                tx::register_operator(),
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

    tx::send_tx_ready(
        api.clone(),
        &tx::create_guild(first_guild).expect("Failed to serialize requirements"),
        Arc::clone(&signer),
    )
    .await
    .expect("failed to create guild");
    tx::send_tx_in_block(
        api.clone(),
        &tx::create_guild(second_guild).expect("Failed to serialize requirements"),
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
            let id_with_auth = RuntimeIdentityWithAuth::Ecdsa(
                RuntimeIdentity::Address20(accounts.eth.address().to_fixed_bytes()),
                RuntimeEcdsaSignature(sig),
            );
            tx::register(id_with_auth, 0)
        })
        .collect::<Vec<_>>();

    let register_discord_payloads = users
        .iter()
        .enumerate()
        .map(|(i, _)| {
            tx::register(
                RuntimeIdentityWithAuth::Other(
                    RuntimeIdentity::Other(discord_id(i as u64)),
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

async fn join_request_tx(
    api: Api,
    guild_name: &GuildName,
    role_name: &RoleName,
    accounts: &Accounts,
) -> Result<Hash, subxt::Error> {
    let tx_payload = tx::manage_role(
        accounts.substrate.account_id().clone(),
        *guild_name,
        *role_name,
    );
    tx::send_tx_in_block(api, &tx_payload, Arc::clone(&accounts.substrate)).await
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
