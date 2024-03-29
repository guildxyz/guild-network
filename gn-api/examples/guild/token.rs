use crate::common::*;
use crate::oracle::*;
use ethers::types::{Address, U256};
use gn_api::{
    tx::{self, Signer},
    Api,
};
use gn_common::filter::Guild as GuildFilter;
use gn_common::identity::{EcdsaSignature, Identity, IdentityWithAuth};
use gn_engine::balance::{Balance, Relation, TokenType};
use gn_engine::chains::EvmChain;
use gn_engine::{Requirement, RequirementsWithLogic};
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

const ADDRESS: &str = "e43878ce78934fe8007748ff481f03b8ee3b97de";
const SIGNATURE: &str = "a7d8263c96a8bb689d462b2782a45b81f02777607c27d1b322a1c46910482e274320fbf353a543a1504dc3c0ded9c2930dffc4b15541d97da7b240f40416f12a1b";

pub async fn token(api: Api, root: Arc<Signer>) {
    let _operators = init_operators(api.clone(), Arc::clone(&root)).await;

    let mut signature = [0u8; 65];
    hex::decode_to_slice(SIGNATURE, &mut signature).expect("this should not fail");
    signature[64] -= 27; // ethereum's eip-115 normalization stuff
    let mut address = [0u8; 20];
    hex::decode_to_slice(ADDRESS, &mut address).expect("this should not fail");
    // register root with test evm address + signature
    let identity = Identity::Address20(address);
    let evm_identity = IdentityWithAuth::Ecdsa(identity, EcdsaSignature(signature));

    let index = 0;
    let tx_payload = tx::register(evm_identity, index);
    tx::send::in_block(api.clone(), &tx_payload, Arc::clone(&root))
        .await
        .expect("failed to register");

    #[cfg(not(feature = "external-oracle"))]
    send_dummy_oracle_answers(api.clone(), &_operators).await;

    wait_for_identity(
        api.clone(),
        root.account_id(),
        &identity,
        usize::from(index),
    )
    .await;

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

    let tx_payload = tx::create_guild(TOKEN_GUILD, vec![1, 2, 3]);
    tx::send::in_block(api.clone(), &tx_payload, Arc::clone(&root))
        .await
        .expect("failed to create guild");

    println!("GUILD CREATED");

    let tx_payload = tx::create_unfiltered_role(TOKEN_GUILD, FIRST_ROLE, first_reqs).unwrap();
    tx::send::in_block(api.clone(), &tx_payload, Arc::clone(&root))
        .await
        .expect("failed to create guild");

    println!("FIRST ROLE CREATED");

    let tx_payload = tx::create_unfiltered_role(TOKEN_GUILD, SECOND_ROLE, second_reqs).unwrap();
    tx::send::in_block(api.clone(), &tx_payload, Arc::clone(&root))
        .await
        .expect("failed to create guild");

    println!("SECOND ROLE CREATED");

    let tx_payload = tx::join(TOKEN_GUILD, FIRST_ROLE, None);
    tx::send::in_block(api.clone(), &tx_payload, Arc::clone(&root))
        .await
        .expect("failed to join guild");

    let guild_filter = GuildFilter {
        name: TOKEN_GUILD,
        role: Some(FIRST_ROLE),
    };

    #[cfg(not(feature = "external-oracle"))]
    send_dummy_oracle_answers(api.clone(), &_operators).await;

    wait_for_members(api.clone(), &guild_filter, 1).await;

    println!("FIRST_ROLE JOINED");

    let tx_payload = tx::join(TOKEN_GUILD, SECOND_ROLE, None);
    tx::send::in_block(api.clone(), &tx_payload, Arc::clone(&root))
        .await
        .expect("failed to join guild");

    let guild_filter = GuildFilter {
        name: TOKEN_GUILD,
        role: Some(SECOND_ROLE),
    };

    #[cfg(not(feature = "external-oracle"))]
    send_dummy_oracle_answers(api.clone(), &_operators).await;

    wait_for_members(api.clone(), &guild_filter, 1).await;

    println!("SECOND_ROLE JOINED");
}
