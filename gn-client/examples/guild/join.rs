use crate::common::*;
use ethers::signers::Signer as EthSigner;
use gn_client::{query, tx::Signer, Api};
use gn_common::filter::Guild as GuildFilter;
use gn_common::identity::Identity;
use gn_common::pad::padded_id;
use gn_test_data::*;
use std::sync::Arc;

const RETRIES: u8 = 10;
const SLEEP_DURATION_MS: u64 = 500;

pub async fn join(api: Api, root: Arc<Signer>) {
    let operators = prefunded_accounts(api.clone(), Arc::clone(&root), N_TEST_ACCOUNTS).await;
    #[cfg(not(feature = "external-oracle"))]
    {
        let registering_operators = operators.values();
        register_operators(api.clone(), Arc::clone(&root), registering_operators).await;
        let registered_operators = query::registered_operators(api.clone())
            .await
            .expect("failed to fetch registered operators");

        for registered in &registered_operators {
            if registered != root.account_id() {
                assert!(operators.get(registered).is_some());
            }
        }
    }

    register_users(api.clone(), &operators).await;

    #[cfg(not(feature = "external-oracle"))]
    send_dummy_oracle_answers(api.clone(), &operators).await;

    // wait for all transactions to be finalized
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

    for (i, (id, accounts)) in operators.iter().enumerate() {
        let user_identity = query::user_identity(api.clone(), id)
            .await
            .expect("failed to fetch individual identity");

        let eth_address = accounts.eth.address().to_fixed_bytes();
        let expected = vec![
            Identity::Address20(eth_address),
            Identity::Other(padded_id(b"discord:", i as u64)),
        ];

        assert_eq!(user_identity, expected);
    }

    create_dummy_guilds(api.clone(), root, operators.values()).await;

    join_guilds(api.clone(), &operators).await;

    #[cfg(not(feature = "external-oracle"))]
    send_dummy_oracle_answers(api.clone(), &operators).await;

    let mut filter = GuildFilter {
        name: FIRST_GUILD,
        role: None,
    };
    loop {
        let members = query::members(api.clone(), &filter, PAGE_SIZE)
            .await
            .expect("failed to fetch members");
        if members.len() == N_TEST_ACCOUNTS {
            println!("FIRST GUILD MEMBERS");
            members
                .into_iter()
                .for_each(|member| println!("\t{member}"));
            break;
        }
    }

    println!("SECOND GUILD MEMBERS");
    filter.name = SECOND_GUILD;
    let members = query::members(api.clone(), &filter, PAGE_SIZE)
        .await
        .expect("failed to fetch members");
    assert_eq!(members.len(), N_TEST_ACCOUNTS);
    members
        .into_iter()
        .for_each(|member| println!("\t{member}"));

    println!("FIRST GUILD FIRST ROLE MEMBERS");
    filter.name = FIRST_GUILD;
    filter.role = Some(FIRST_ROLE);
    let members = query::members(api.clone(), &filter, PAGE_SIZE)
        .await
        .expect("failed to fetch members");
    assert_eq!(members.len(), N_TEST_ACCOUNTS);
    members
        .into_iter()
        .for_each(|member| println!("\t{member}"));

    println!("FIRST GUILD SECOND ROLE MEMBERS");
    filter.role = Some(SECOND_ROLE);
    let members = query::members(api.clone(), &filter, PAGE_SIZE)
        .await
        .expect("failed to fetch members");
    assert_eq!(members.len(), 2);
    members
        .into_iter()
        .for_each(|member| println!("\t{member}"));

    println!("SECOND GUILD FIRST ROLE MEMBERS");
    filter.name = SECOND_GUILD;
    filter.role = Some(FIRST_ROLE);
    let members = query::members(api.clone(), &filter, PAGE_SIZE)
        .await
        .expect("failed to fetch members");
    assert_eq!(members.len(), 5);
    members
        .into_iter()
        .for_each(|member| println!("\t{member}"));

    println!("SECOND GUILD SECOND ROLE MEMBERS");
    filter.role = Some(SECOND_ROLE);
    let members = query::members(api.clone(), &filter, PAGE_SIZE)
        .await
        .expect("failed to fetch members");
    assert_eq!(members.len(), 5);
    members
        .into_iter()
        .for_each(|member| println!("\t{member}"));
}
