mod common;
use common::*;

use ethers::signers::Signer as EthSigner;
use gn_client::queries::*;
use gn_common::identities::Identity;
use std::sync::Arc;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Client params",
    about = "Advanced parameters for the Substrate client."
)]
struct Opt {
    /// Set node IP address
    #[structopt(short = "i", long = "node-ip", default_value = "127.0.0.1")]
    node_ip: String,

    /// Set node port number
    #[structopt(short = "p", long = "node-port", default_value = "9944")]
    node_port: String,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();

    let url = format!("ws://{}:{}", opt.node_ip, opt.node_port);

    let (api, alice) = api_with_alice(url).await;

    let operators = prefunded_accounts(api.clone(), Arc::clone(&alice), N_TEST_ACCOUNTS).await;

    #[cfg(not(feature = "external-oracle"))]
    {
        let registering_operators = operators.values();
        register_operators(api.clone(), registering_operators).await;
        let registered_operators = registered_operators(api.clone())
            .await
            .expect("failed to fetch registered operators");
        for registered in &registered_operators {
            assert!(operators.get(registered).is_some());
        }
    }

    register_users(api.clone(), &operators).await;

    #[cfg(not(feature = "external-oracle"))]
    send_dummy_oracle_answers(api.clone(), &operators).await;

    loop {
        let user_identities = user_identities(api.clone(), PAGE_SIZE)
            .await
            .expect("failed to fetch user identities");
        if user_identities.len() == N_TEST_ACCOUNTS {
            for (i, (id, accounts)) in operators.iter().enumerate() {
                let user_identity = user_identity(api.clone(), id)
                    .await
                    .expect("failed to fetch individual identity");

                let expected = &[
                    Identity::EvmChain(accounts.eth.address().to_fixed_bytes()),
                    Identity::Discord(i as u64),
                ];
                assert_eq!(user_identities.get(id).unwrap(), expected);
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
        let all_members = members(api.clone(), None, PAGE_SIZE)
            .await
            .expect("failed to fetch registered members");
        if all_members.len() == N_TEST_ACCOUNTS {
            println!("ALL MEMBERS");
            println!("{all_members:#?}");
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
    println!("{first_guild_members:#?}");

    filter.name = SECOND_GUILD;
    let second_guild_members = members(api.clone(), Some(&filter), PAGE_SIZE)
        .await
        .expect("failed to fetch members");
    println!("SECOND GUILD MEMBERS");
    println!("{second_guild_members:#?}");

    filter.name = FIRST_GUILD;
    filter.role = Some(FIRST_ROLE);
    let first_guild_first_role_members = members(api.clone(), Some(&filter), PAGE_SIZE)
        .await
        .expect("failed to fetch members");
    println!("FIRST GUILD FIRST ROLE MEMBERS");
    println!("{first_guild_first_role_members:#?}");

    filter.role = Some(SECOND_ROLE);
    let first_guild_second_role_members = members(api.clone(), Some(&filter), PAGE_SIZE)
        .await
        .expect("failed to fetch members");
    println!("FIRST GUILD SECOND ROLE MEMBERS");
    println!("{first_guild_second_role_members:#?}");

    filter.name = SECOND_GUILD;
    filter.role = Some(FIRST_ROLE);
    let second_guild_first_role_members = members(api.clone(), Some(&filter), PAGE_SIZE)
        .await
        .expect("failed to fetch members");
    println!("SECOND GUILD FIRST ROLE MEMBERS");
    println!("{second_guild_first_role_members:#?}");

    filter.role = Some(SECOND_ROLE);
    let second_guild_second_role_members = members(api.clone(), Some(&filter), PAGE_SIZE)
        .await
        .expect("failed to fetch members");
    println!("SECOND GUILD SECOND ROLE MEMBERS");
    println!("{second_guild_second_role_members:#?}");
}
