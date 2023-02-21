use crate::common::*;
use ethers::signers::Signer as EthSigner;
use gn_client::{query, tx::Signer, Api};
use gn_common::filter::Guild as GuildFilter;
use gn_common::identity::Identity;
use gn_common::pad::padded_id;
use gn_test_data::*;
use std::sync::Arc;

pub async fn join(api: Api, root: Arc<Signer>) {
    let operators = prefunded_accounts(api.clone(), Arc::clone(&root), N_TEST_ACCOUNTS).await;
    #[cfg(not(feature = "external-oracle"))]
    {
        register_operators(api.clone(), Arc::clone(&root), operators.values()).await;
        activate_operators(api.clone(), operators.values()).await;
        let active_operators = query::active_operators(api.clone())
            .await
            .expect("failed to fetch active operators");

        for active in &active_operators {
            assert!(operators.get(active).is_some());
        }
    }

    #[cfg(not(feature = "external-oracle"))]
    {
        wait_for_active_operator(api.clone()).await;
    }

    register_users(api.clone(), &operators).await;

    #[cfg(not(feature = "external-oracle"))]
    send_dummy_oracle_answers(api.clone(), &operators).await;

    // wait for all transactions to be finalized
    wait_for_oracle_answers(api.clone()).await;

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
