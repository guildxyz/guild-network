use crate::common::*;
use ethers::signers::Signer as EthSigner;
use gn_client::{query, tx::Signer, Api};
use gn_common::identity::Identity;
use gn_test_data::*;
use std::sync::Arc;

pub async fn join(api: Api, alice: Arc<Signer>) {
    let operators = prefunded_accounts(api.clone(), Arc::clone(&alice), N_TEST_ACCOUNTS).await;

    #[cfg(not(feature = "external-oracle"))]
    {
        let registering_operators = operators.values();
        register_operators(api.clone(), registering_operators).await;
        let registered_operators = query::registered_operators(api.clone())
            .await
            .expect("failed to fetch registered operators");

        for registered in &registered_operators {
            if registered != alice.account_id() {
                assert!(operators.get(registered).is_some());
            }
        }
    }

    register_users(api.clone(), &operators).await;

    #[cfg(not(feature = "external-oracle"))]
    send_dummy_oracle_answers(api.clone(), &operators).await;

    loop {
        let user_identities = query::user_identities(api.clone(), PAGE_SIZE)
            .await
            .expect("failed to fetch user identities");
        if user_identities.len() == N_TEST_ACCOUNTS {
            for (i, (id, accounts)) in operators.iter().enumerate() {
                let user_identity = query::user_identity(api.clone(), id)
                    .await
                    .expect("failed to fetch individual identity");

                let eth_address = accounts.eth.address().to_fixed_bytes();
                let expected = vec![
                    (0, Identity::Address20(eth_address)),
                    (1, Identity::Other(discord_id(i as u64))),
                ]
                .into_iter()
                .collect();
                assert_eq!(user_identities.get(id).unwrap(), &expected);
                assert_eq!(user_identity, expected);
            }
            println!("USER IDENTITIES MATCH");
            break;
        }
    }

    create_dummy_guilds(api.clone(), alice, operators.values()).await;

    join_guilds(api.clone(), &operators).await;

    #[cfg(not(feature = "external-oracle"))]
    send_dummy_oracle_answers(api.clone(), &operators).await;

    loop {
        let all_members = query::members(api.clone(), None, PAGE_SIZE)
            .await
            .expect("failed to fetch registered members");
        if all_members.len() == N_TEST_ACCOUNTS {
            println!("ALL MEMBERS");
            all_members
                .into_iter()
                .for_each(|member| println!("\t{member}"));
            break;
        }
    }

    let mut filter = query::GuildFilter {
        name: FIRST_GUILD,
        role: None,
    };
    println!("FIRST GUILD MEMBERS");
    query::members(api.clone(), Some(&filter), PAGE_SIZE)
        .await
        .expect("failed to fetch members")
        .into_iter()
        .for_each(|member| println!("\t{member}"));

    println!("SECOND GUILD MEMBERS");
    filter.name = SECOND_GUILD;
    query::members(api.clone(), Some(&filter), PAGE_SIZE)
        .await
        .expect("failed to fetch members")
        .into_iter()
        .for_each(|member| println!("\t{member}"));

    println!("FIRST GUILD FIRST ROLE MEMBERS");
    filter.name = FIRST_GUILD;
    filter.role = Some(FIRST_ROLE);
    query::members(api.clone(), Some(&filter), PAGE_SIZE)
        .await
        .expect("failed to fetch members")
        .into_iter()
        .for_each(|member| println!("\t{member}"));

    println!("FIRST GUILD SECOND ROLE MEMBERS");
    filter.role = Some(SECOND_ROLE);
    query::members(api.clone(), Some(&filter), PAGE_SIZE)
        .await
        .expect("failed to fetch members")
        .into_iter()
        .for_each(|member| println!("\t{member}"));

    println!("SECOND GUILD FIRST ROLE MEMBERS");
    filter.name = SECOND_GUILD;
    filter.role = Some(FIRST_ROLE);
    query::members(api.clone(), Some(&filter), PAGE_SIZE)
        .await
        .expect("failed to fetch members")
        .into_iter()
        .for_each(|member| println!("\t{member}"));

    println!("SECOND GUILD SECOND ROLE MEMBERS");
    filter.role = Some(SECOND_ROLE);
    query::members(api.clone(), Some(&filter), PAGE_SIZE)
        .await
        .expect("failed to fetch members")
        .into_iter()
        .for_each(|member| println!("\t{member}"));
}
