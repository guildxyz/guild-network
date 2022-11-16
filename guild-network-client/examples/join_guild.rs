use guild_network_client::queries::*;
use guild_network_client::transactions::*;
use guild_network_client::{pad_to_32_bytes, runtime, Api, Guild, Role, Signer};
use guild_network_gate::requirements::Requirement;
use sp_keyring::AccountKeyring;

use std::sync::Arc;

use runtime::runtime_types::pallet_guild::pallet::Call as GuildCall;

const URL: &str = "ws://127.0.0.1:9944";
const PAGE_SIZE: u32 = 10;

#[tokio::main]
async fn main() {
    let api = Api::from_url(URL)
        .await
        .expect("failed to initialize client");
    let signer = Arc::new(Signer::new(AccountKeyring::Alice.pair()));

    // register signer as oracle operator
    let tx = register_operator();
    let hash = send_tx_in_block(api.clone(), tx, Arc::clone(&signer))
        .await
        .unwrap();
    println!("Operator registered: {}", hash);

    let role_name = pad_to_32_bytes(b"myrole").unwrap();
    let guild_name = pad_to_32_bytes(b"myguild").unwrap();

    let role = Role {
        name: role_name,
        requirements: vec![Requirement::Free],
    };
    let guild = Guild {
        name: guild_name,
        metadata: vec![1, 2, 3],
        roles: vec![role],
    };

    let hash = send_tx_in_block(api.clone(), create_guild(guild), Arc::clone(&signer))
        .await
        .unwrap();
    println!("Guild created: {}", hash);

    let tx = join_guild(guild_name, role_name, vec![], vec![]);
    let hash = send_tx_in_block(api.clone(), tx, Arc::clone(&signer))
        .await
        .unwrap();
    println!("Join request submitted: {}", hash);

    join_requests(api.clone(), PAGE_SIZE).await.unwrap();

    let tx = oracle_init_request(
        0,
        vec![1, 2, 3],
        100_000_000,
        GuildCall::callback {
            expired: false,
            result: vec![],
        },
    );
    send_tx_in_block(api.clone(), tx, Arc::clone(&signer))
        .await
        .unwrap();

    println!("ORACLE REQUESTS");
    oracle_requests(api.clone(), PAGE_SIZE).await.unwrap();
    let tx = oracle_callback(0, vec![1]);
    let hash = send_tx_in_block(api.clone(), tx, Arc::clone(&signer))
        .await
        .unwrap();
    println!("Oracle callback sent: {}", hash);

    println!("MEMBERS");
    let members = members(api, PAGE_SIZE).await.unwrap();
    println!("{:?}", members);
}
