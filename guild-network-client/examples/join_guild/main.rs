mod common;
use common::*;

use guild_network_client::queries::*;
#[cfg(feature = "external-oracle")]
use guild_network_client::transactions::{join_guild, send_tx_in_block};
use guild_network_client::{AccountId, Api, Signer};

use std::collections::BTreeMap;
use std::sync::Arc;

const N_TEST_ACCOUNTS: usize = 10;

#[tokio::main]
async fn main() {
    let (api, alice) = api_with_alice().await;
    let operators = prefunded_accounts(api.clone(), Arc::clone(&alice), N_TEST_ACCOUNTS).await;
    register_operators(api.clone(), &operators).await;

    let registered_operators = registered_operators(api.clone())
        .await
        .expect("failed to fetch registered operators");

    for registered in &registered_operators {
        assert!(operators.get(registered).is_some());
    }

    create_dummy_guilds(api.clone(), alice).await;

    join_guild_sim(api, operators).await;
}

#[cfg(not(feature = "external-oracle"))]
async fn join_guild_sim(api: Api, operators: BTreeMap<AccountId, Arc<Signer>>) {
    join_guilds(api.clone(), &operators).await;

    send_dummy_oracle_answers(api.clone(), &operators).await;

    let all_members = members(api.clone(), None, 10)
        .await
        .expect("failed to fetch registered members");
    println!("ALL MEMBERS");
    println!("{:#?}", all_members);

    let mut filter = GuildFilter {
        name: FIRST_GUILD,
        role: None,
    };
    let first_guild_members = members(api.clone(), Some(&filter), 10)
        .await
        .expect("failed to fetch members");
    println!("FIRST GUILD MEMBERS");
    println!("{:#?}", first_guild_members);

    filter.name = SECOND_GUILD;
    let second_guild_members = members(api.clone(), Some(&filter), 10)
        .await
        .expect("failed to fetch members");
    println!("SECOND GUILD MEMBERS");
    println!("{:#?}", second_guild_members);

    filter.name = FIRST_GUILD;
    filter.role = Some(FIRST_ROLE);
    let first_guild_first_role_members = members(api.clone(), Some(&filter), 10)
        .await
        .expect("failed to fetch members");
    println!("FIRST GUILD FIRST ROLE MEMBERS");
    println!("{:#?}", first_guild_first_role_members);

    filter.role = Some(SECOND_ROLE);
    let first_guild_second_role_members = members(api.clone(), Some(&filter), 10)
        .await
        .expect("failed to fetch members");
    println!("FIRST GUILD SECOND ROLE MEMBERS");
    println!("{:#?}", first_guild_second_role_members);

    filter.name = SECOND_GUILD;
    filter.role = Some(FIRST_ROLE);
    let second_guild_first_role_members = members(api.clone(), Some(&filter), 10)
        .await
        .expect("failed to fetch members");
    println!("SECOND GUILD FIRST ROLE MEMBERS");
    println!("{:#?}", second_guild_first_role_members);

    filter.role = Some(SECOND_ROLE);
    let second_guild_second_role_members = members(api.clone(), Some(&filter), 10)
        .await
        .expect("failed to fetch members");
    println!("SECOND GUILD SECOND ROLE MEMBERS");
    println!("{:#?}", second_guild_second_role_members);
}

#[cfg(feature = "external-oracle")]
async fn join_guild_sim(api: Api, operators: BTreeMap<AccountId, Arc<Signer>>) {
    let (id, signer) = operators.into_iter().next().unwrap();
    let tx = join_guild(FIRST_GUILD, SECOND_ROLE, vec![], vec![]);
    let hash = send_tx_in_block(api.clone(), &tx, Arc::clone(&signer))
        .await
        .unwrap();
    println!("Join request submitted: {}", hash);

    loop {
        let members = members(api.clone(), None, PAGE_SIZE).await.unwrap();
        if !members.is_empty() {
            println!("MEMBERS");
            println!("{:?}", members);
            assert_eq!(members[0], id);
            break;
        }
    }
}
