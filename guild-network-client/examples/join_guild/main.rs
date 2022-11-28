mod common;
use common::*;

use ethers::signers::Signer as EthSigner;
use guild_network_client::queries::*;
use guild_network_gate::identities::Identity;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let (api, alice) = api_with_alice().await;

    let operators = prefunded_accounts(api.clone(), Arc::clone(&alice), N_TEST_ACCOUNTS).await;

    #[cfg(not(feature = "external-oracle"))]
    let registering_operators = operators.values();
    #[cfg(feature = "external-oracle")]
    let alice_vec = vec![Accounts {
        substrate: Arc::clone(&alice),
        eth: ethers::signers::LocalWallet::new(&mut rand::rngs::OsRng),
    }];
    #[cfg(feature = "external-oracle")]
    let registering_operators = alice_vec.iter();

    register_operators(api.clone(), registering_operators).await;

    #[cfg(not(feature = "external-oracle"))]
    let registered_operators = registered_operators(api.clone())
        .await
        .expect("failed to fetch registered operators");

    #[cfg(not(feature = "external-oracle"))]
    for registered in &registered_operators {
        assert!(operators.get(registered).is_some());
    }

    create_dummy_guilds(api.clone(), alice, operators.values()).await;

    join_guilds(api.clone(), &operators).await;

    #[cfg(not(feature = "external-oracle"))]
    send_dummy_oracle_answers(api.clone(), &operators).await;

    loop {
        let all_members = members(api.clone(), None, PAGE_SIZE)
            .await
            .expect("failed to fetch registered members");
        if all_members.len() == N_TEST_ACCOUNTS {
            println!("ALL MEMBERS");
            println!("{:#?}", all_members);
            break;
        }
    }

    let mut filter = GuildFilter {
        name: FIRST_GUILD,
        role: None,
    };
    let first_guild_members = members(api.clone(), Some(&filter), PAGE_SIZE)
        .await
        .expect("failed to fetch members");
    println!("FIRST GUILD MEMBERS");
    println!("{:#?}", first_guild_members);

    filter.name = SECOND_GUILD;
    let second_guild_members = members(api.clone(), Some(&filter), PAGE_SIZE)
        .await
        .expect("failed to fetch members");
    println!("SECOND GUILD MEMBERS");
    println!("{:#?}", second_guild_members);

    filter.name = FIRST_GUILD;
    filter.role = Some(FIRST_ROLE);
    let first_guild_first_role_members = members(api.clone(), Some(&filter), PAGE_SIZE)
        .await
        .expect("failed to fetch members");
    println!("FIRST GUILD FIRST ROLE MEMBERS");
    println!("{:#?}", first_guild_first_role_members);

    filter.role = Some(SECOND_ROLE);
    let first_guild_second_role_members = members(api.clone(), Some(&filter), PAGE_SIZE)
        .await
        .expect("failed to fetch members");
    println!("FIRST GUILD SECOND ROLE MEMBERS");
    println!("{:#?}", first_guild_second_role_members);

    filter.name = SECOND_GUILD;
    filter.role = Some(FIRST_ROLE);
    let second_guild_first_role_members = members(api.clone(), Some(&filter), PAGE_SIZE)
        .await
        .expect("failed to fetch members");
    println!("SECOND GUILD FIRST ROLE MEMBERS");
    println!("{:#?}", second_guild_first_role_members);

    filter.role = Some(SECOND_ROLE);
    let second_guild_second_role_members = members(api.clone(), Some(&filter), PAGE_SIZE)
        .await
        .expect("failed to fetch members");
    println!("SECOND GUILD SECOND ROLE MEMBERS");
    println!("{:#?}", second_guild_second_role_members);

    let user_identities = user_identities(api, PAGE_SIZE)
        .await
        .expect("failed to load user ids");
    for (account, ids) in user_identities.iter() {
        for id in ids {
            let expected_address = operators.get(account).unwrap().eth.address();
            match id {
                Identity::EvmChain(address) => {
                    assert_eq!(address.as_bytes(), expected_address.as_bytes())
                }
                _ => unimplemented!(),
            }
        }
    }
}
