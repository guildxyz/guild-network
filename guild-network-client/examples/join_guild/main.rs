mod common;
use common::*;

use guild_network_client::queries::*;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let (api, alice) = api_with_alice().await;
    let operators = prefunded_accounts(api.clone(), Arc::clone(&alice)).await;
    register_operators(api.clone(), operators.values()).await;

    let registered_operators = registered_operators(api.clone())
        .await
        .expect("failed to fetch registered operators");

    for registered in &registered_operators {
        assert!(operators.get(registered).is_some());
    }

    create_dummy_guilds(api.clone(), alice).await;

    join_guilds(api.clone(), operators.values()).await;

    send_dummy_oracle_answers(api, &operators).await;
}
