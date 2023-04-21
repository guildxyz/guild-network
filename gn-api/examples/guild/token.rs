use crate::common::*;
use crate::oracle::*;
use gn_api::{
    tx::{self, Signer},
    Api,
};
use gn_common::filter::Guild as GuildFilter;
use gn_common::identity::{EcdsaSignature, Identity, IdentityWithAuth};
use gn_test_data::*;

use std::sync::Arc;

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

    let (first_reqs, second_reqs) = dummy_requirements_with_logic();

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
