use ethers::signers::{LocalWallet, Signer as EthSigner};
use futures::future::try_join_all;
use guild_network_client::data::*;
#[cfg(not(feature = "external-oracle"))]
use guild_network_client::queries::*;
use guild_network_client::transactions::*;
use guild_network_client::{AccountId, Api, Hash, Keypair, Signer, TxStatus};
use guild_network_common::{GuildName, RoleName};
use guild_network_gate::identities::{Identity, IdentityAuth};
use guild_network_gate::requirements::Requirement;
use guild_network_gate::{EvmAddress, EvmSignature};
use rand::{rngs::StdRng, SeedableRng};
use sp_keyring::AccountKeyring;
use subxt::ext::sp_core::crypto::Pair as TraitPair;

use std::collections::BTreeMap;
use std::sync::Arc;

const URL: &str = "ws://127.0.0.1:9944";
pub const FIRST_ROLE: RoleName = [0; 32];
pub const SECOND_ROLE: RoleName = [1; 32];
pub const FIRST_GUILD: GuildName = [2; 32];
pub const SECOND_GUILD: GuildName = [3; 32];
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
        .map(|acc| EvmAddress::from_slice(acc.eth.address().as_bytes()))
        .collect();
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

pub async fn join_guilds(api: Api, operators: &BTreeMap<AccountId, Accounts>) {
    let join_request_futures = operators
        .iter()
        .enumerate()
        .map(|(i, (id, accounts))| match i % 4 {
            0 => join_request_tx(api.clone(), &FIRST_GUILD, &FIRST_ROLE, id, accounts),
            1 => join_request_tx(api.clone(), &FIRST_GUILD, &SECOND_ROLE, id, accounts),
            2 => join_request_tx(api.clone(), &SECOND_GUILD, &FIRST_ROLE, id, accounts),
            3 => join_request_tx(api.clone(), &SECOND_GUILD, &SECOND_ROLE, id, accounts),
            _ => unreachable!(),
        })
        .collect::<Vec<_>>();

    try_join_all(join_request_futures)
        .await
        .expect("failed to submit oracle answers");

    println!("join requests successfully submitted");
}

async fn join_request_tx(
    api: Api,
    guild_name: &GuildName,
    role_name: &RoleName,
    id: &AccountId,
    accounts: &Accounts,
) -> Result<Hash, subxt::Error> {
    let msg = guild_network_client::signed_msg(id, guild_name, role_name);
    let signature = accounts
        .eth
        .sign_message(&msg)
        .await
        .map_err(|e| subxt::Error::Other(e.to_string()))?;
    let tx_payload = join_guild(
        *guild_name,
        *role_name,
        vec![Identity::EvmChain(EvmAddress::from_slice(
            accounts.eth.address().as_bytes(),
        ))],
        vec![IdentityAuth::EvmChain {
            signature: EvmSignature::from_slice(&signature.to_vec()),
            msg: msg.as_bytes().to_vec(),
        }],
    )
    .expect("Failed to serialize data");
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
