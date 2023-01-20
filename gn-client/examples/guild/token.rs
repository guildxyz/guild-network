#[cfg(not(feature = "external-oracle"))]
use crate::common::*;
use ethers::types::{Address, U256};
use gn_client::data::{Guild, Role};
use gn_client::{queries, transactions, Api, RuntimeIdentityWithAuth, Signer};
use gn_common::identities::Identity;
use gn_common::requirements::balance::{Relation, RequiredBalance, TokenType};
use gn_common::requirements::chains::EvmChain;
use gn_common::requirements::{Requirement, RequirementsWithLogic};
use gn_test_data::*;

use std::str::FromStr;
use std::sync::Arc;

const ETH_ERC721_ADDRESS: &str = "57f1887a8bf19b14fc0df6fd9b2acc9af147ea85";
const ETH_ERC721_ID: &str =
    "61313325075603536901663283754390960556726744542208800735045237225934362163454";

const GNOSIS_ERC721_ADDRESS_0: &str = "22c1f6050e56d2876009903609a2cc3fef83b415";
const GNOSIS_ERC721_ID_0: &str = "5752323";

const GNOSIS_ERC721_ADDRESS_1: &str = "22c1f6050e56d2876009903609a2cc3fef83b415";
const GNOSIS_ERC721_ID_1: &str = "5819774";

// NOTE this needs an external oracle to be running
pub async fn token(api: Api, alice: Arc<Signer>) {
    let mut signature = [0u8; 65];
    hex::decode_to_slice(
"cfc5dd009163cc4d884946f0ccae5ea3a37794337b64cf5f076e6cd4c2af81a8727e044672704ce6026d6a440527943fccd9c044f7398c892c75090a1b0cadb71c", &mut  signature).expect("this should not fail");
    let mut address = [0u8; 20];
    hex::decode_to_slice("e43878ce78934fe8007748ff481f03b8ee3b97de", &mut address)
        .expect("this should not fail");

    #[cfg(not(feature = "external-oracle"))]
    let operators = prefunded_accounts(api.clone(), Arc::clone(&alice), N_TEST_ACCOUNTS).await;

    #[cfg(not(feature = "external-oracle"))]
    {
        let registering_operators = operators.values();
        register_operators(api.clone(), registering_operators).await;
        let registered_operators = queries::registered_operators(api.clone())
            .await
            .expect("failed to fetch registered operators");

        for registered in &registered_operators {
            if registered != alice.account_id() {
                assert!(operators.get(registered).is_some());
            }
        }
    }

    // register alice with test evm address + signature
    let evm_identity = RuntimeIdentityWithAuth::EvmChain(address, signature);

    let tx_payload = transactions::register(vec![evm_identity]);
    transactions::send_tx_in_block(api.clone(), &tx_payload, Arc::clone(&alice))
        .await
        .expect("failed to register");

    #[cfg(not(feature = "external-oracle"))]
    send_dummy_oracle_answers(api.clone(), &operators).await;

    loop {
        let user_identity = queries::user_identity(api.clone(), alice.account_id())
            .await
            .expect("failed to fetch user identities");
        if user_identity.len() == 1 {
            assert_eq!(user_identity, &[Identity::EvmChain(address)]);
            break;
        }
    }

    println!("USER REGISTERED");

    let mut one = [0u8; 32];
    one[0] = 1;

    let roles = vec![
        Role {
            name: FIRST_ROLE,
            reqs: RequirementsWithLogic {
                logic: "0 AND 1".to_string(),
                requirements: vec![
                    Requirement::EvmBalance(RequiredBalance {
                        token_type: Some(TokenType::NonFungible {
                            address: Address::from_str(ETH_ERC721_ADDRESS)
                                .unwrap()
                                .to_fixed_bytes(),
                            id: U256::from_dec_str(ETH_ERC721_ID).unwrap().into(),
                        }),
                        relation: Relation::EqualTo(one),
                        chain: EvmChain::Ethereum,
                    }),
                    Requirement::EvmBalance(RequiredBalance {
                        token_type: None,
                        relation: Relation::GreaterThan([0u8; 32]),
                        chain: EvmChain::Ethereum,
                    }),
                ],
            },
        },
        Role {
            name: SECOND_ROLE,
            reqs: RequirementsWithLogic {
                logic: "0 OR (1 AND 2)".to_string(),
                requirements: vec![
                    Requirement::EvmBalance(RequiredBalance {
                        token_type: None,
                        relation: Relation::GreaterThan([1u8; 32]),
                        chain: EvmChain::Ethereum,
                    }),
                    Requirement::EvmBalance(RequiredBalance {
                        token_type: Some(TokenType::NonFungible {
                            address: Address::from_str(GNOSIS_ERC721_ADDRESS_0)
                                .unwrap()
                                .to_fixed_bytes(),
                            id: U256::from_dec_str(GNOSIS_ERC721_ID_0).unwrap().into(),
                        }),
                        relation: Relation::EqualTo(one),
                        chain: EvmChain::Gnosis,
                    }),
                    Requirement::EvmBalance(RequiredBalance {
                        token_type: Some(TokenType::NonFungible {
                            address: Address::from_str(GNOSIS_ERC721_ADDRESS_1)
                                .unwrap()
                                .to_fixed_bytes(),
                            id: U256::from_dec_str(GNOSIS_ERC721_ID_1).unwrap().into(),
                        }),
                        relation: Relation::EqualTo(U256::from(1).into()),
                        chain: EvmChain::Gnosis,
                    }),
                ],
            },
        },
    ];

    let guild = Guild {
        name: TOKEN_GUILD,
        metadata: vec![1, 2, 3],
        roles,
    };

    let tx_payload = transactions::create_guild(guild).expect("failed to serialize requirements");
    transactions::send_tx_in_block(api.clone(), &tx_payload, Arc::clone(&alice))
        .await
        .expect("failed to create guild");

    println!("GUILD CREATED");

    let tx_payload = transactions::manage_role(alice.account_id().clone(), TOKEN_GUILD, FIRST_ROLE);
    transactions::send_tx_in_block(api.clone(), &tx_payload, Arc::clone(&alice))
        .await
        .expect("failed to join guild");

    let guild_filter = queries::GuildFilter {
        name: TOKEN_GUILD,
        role: Some(FIRST_ROLE),
    };

    #[cfg(not(feature = "external-oracle"))]
    send_dummy_oracle_answers(api.clone(), &operators).await;

    loop {
        let members = queries::members(api.clone(), Some(&guild_filter), PAGE_SIZE)
            .await
            .expect("failed to query members");
        if members.len() == 1 {
            assert_eq!(members.get(0).unwrap(), alice.account_id());
            break;
        }
    }

    println!("FIRST_ROLE JOINED");

    let tx_payload =
        transactions::manage_role(alice.account_id().clone(), TOKEN_GUILD, SECOND_ROLE);
    transactions::send_tx_in_block(api.clone(), &tx_payload, Arc::clone(&alice))
        .await
        .expect("failed to join guild");

    let guild_filter = queries::GuildFilter {
        name: TOKEN_GUILD,
        role: Some(SECOND_ROLE),
    };

    #[cfg(not(feature = "external-oracle"))]
    send_dummy_oracle_answers(api.clone(), &operators).await;

    loop {
        let members = queries::members(api.clone(), Some(&guild_filter), PAGE_SIZE)
            .await
            .expect("failed to query members");
        if members.len() == 1 {
            assert_eq!(members.get(0).unwrap(), alice.account_id());
            break;
        }
    }

    println!("SECOND_ROLE JOINED");
}
