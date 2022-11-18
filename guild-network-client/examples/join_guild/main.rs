mod common;
use common::*;

use guild_network_client::queries::*;
use std::sync::Arc;

const N_TEST_ACCOUNTS: usize = 10;

#[tokio::main]
async fn main() {
    let (api, alice) = api_with_alice().await;
    /*
    let operators = prefunded_accounts(api.clone(), Arc::clone(&alice), N_TEST_ACCOUNTS).await;
    register_operators(api.clone(), &operators).await;

    let registered_operators = registered_operators(api.clone())
        .await
        .expect("failed to fetch registered operators");

    for registered in &registered_operators {
        assert!(operators.get(registered).is_some());
    }

    create_dummy_guilds(api.clone(), alice).await;

    join_guilds(api.clone(), &operators).await;

    send_dummy_oracle_answers(api.clone(), &operators).await;
    */

    let members = members(api.clone(), 10)
        .await
        .expect("failed to fetch registered members");
    println!("MEMBERS");
    println!("{:#?}", members);

    //for member in members.values() {
    //    assert!(operators.get(member).is_some());
    //}

    let role_ids = role_id(api.clone(), FIRST_GUILD, None, 10)
        .await
        .expect("failed to fetch role ids");
    println!("{:#?}", role_ids);
    let single_role_id = role_id(api, FIRST_GUILD, Some(SECOND_ROLE), 10)
        .await
        .expect("failed to fetch role ids");
    println!("{:#?}", single_role_id);
}
