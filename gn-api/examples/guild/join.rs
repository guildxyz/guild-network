use crate::common::*;
use crate::oracle::*;
use gn_api::{
    query,
    tx::{Signer, SignerT},
    Api,
};
use gn_common::filter::Guild as GuildFilter;
use gn_common::identity::Identity;
use gn_common::pad::padded_id;
use gn_test_data::*;
use std::sync::Arc;

pub async fn join(api: Api, root: Arc<Signer>) {
    let _operators = init_operators(api.clone(), Arc::clone(&root)).await;

    let users = dummy_users();

    register_users(api.clone(), &users).await;

    #[cfg(not(feature = "external-oracle"))]
    send_dummy_oracle_answers(api.clone(), &_operators).await;

    // wait for all transactions to be finalized
    wait_for_oracle_answers(api.clone()).await;

    for (i, acc) in users.iter().enumerate() {
        let user_identity = query::user_identity(api.clone(), acc.account_id())
            .await
            .expect("failed to fetch individual identity");

        let eth_address = acc.evm_address();
        let expected = vec![
            Identity::Address20(eth_address),
            Identity::Other(padded_id(b"discord:", i as u64)),
        ];

        assert_eq!(user_identity, expected);
    }

    create_dummy_guilds(api.clone(), root, &users).await;

    join_guilds(api.clone(), &users).await;

    #[cfg(not(feature = "external-oracle"))]
    send_dummy_oracle_answers(api.clone(), &_operators).await;

    let mut filter = GuildFilter {
        name: FIRST_GUILD,
        role: None,
    };

    wait_for_members(api.clone(), &filter, N_TEST_ACCOUNTS).await;

    println!("SECOND GUILD MEMBERS");
    filter.name = SECOND_GUILD;
    wait_for_members(api.clone(), &filter, N_TEST_ACCOUNTS).await;

    println!("FIRST GUILD FIRST ROLE MEMBERS");
    filter.name = FIRST_GUILD;
    filter.role = Some(FIRST_ROLE);
    wait_for_members(api.clone(), &filter, N_TEST_ACCOUNTS).await;

    println!("FIRST GUILD SECOND ROLE MEMBERS");
    filter.role = Some(SECOND_ROLE);
    wait_for_members(api.clone(), &filter, 2).await;

    println!("SECOND GUILD FIRST ROLE MEMBERS");
    filter.name = SECOND_GUILD;
    filter.role = Some(FIRST_ROLE);
    wait_for_members(api.clone(), &filter, 5).await;

    println!("SECOND GUILD SECOND ROLE MEMBERS");
    filter.role = Some(SECOND_ROLE);
    wait_for_members(api.clone(), &filter, 5).await;
}
