#![allow(legacy_derive_helpers)]

pub mod data;

#[macro_export]
macro_rules! address {
    ($addr:expr) => {{
        use std::str::FromStr;
        ethers::types::Address::from_str($addr).expect(&format!("Invalid address {}", $addr))
    }};
}

#[derive(Clone, Copy)]
pub enum Chain {
    Ethereum = 1,
    Bsc = 56,
    Gnosis = 100,
    Polygon = 137,
}

pub async fn check_requirement(
    client: &ReqwestClient,
    requriement: &Requirement,
    user_identity: &Identity,
) -> Result<bool, anyhow::Error> {
    match (requirement, user_identity) {
        (Self::Free, _) => Ok(true),
        (Self::EthereumBalance(req_balance), Identity::EvmChain(user_address)) => {
            let balance = get_evm_balance(
                client,
                &req_balance.token_type,
                user_address,
                Chain::Ethereum,
            )
            .await?;
            Ok(req_balance.relation.assert(&balance, &req_balance.amount))
        }
        (Self::BscBalance(req_balance), Identity::EvmChain(user_address)) => {
            let balance =
                get_evm_balance(client, &req_balance.token_type, user_address, Chain::Bsc).await?;
            Ok(req_balance.relation.assert(&balance, &req_balance.amount))
        }
        (Self::GnosisBalance(req_balance), Identity::EvmChain(user_address)) => {
            let balance =
                get_evm_balance(client, &req_balance.token_type, user_address, Chain::Gnosis)
                    .await?;
            Ok(req_balance.relation.assert(&balance, &req_balance.amount))
        }
        (Self::PolygonBalance(req_balance), Identity::EvmChain(user_address)) => {
            let balance = get_evm_balance(
                client,
                &req_balance.token_type,
                user_address,
                Chain::Polygon,
            )
            .await?;
            Ok(req_balance.relation.assert(&balance, &req_balance.amount))
        }
        (Self::EvmAllowlist(allowlist), Identity::EvmChain(user_address)) => {
            Ok(allowlist.is_member(user_address))
        }
        _ => Ok(false),
    }
}

pub async fn process_request<'a>(
    request_id: u64,
    minimum_balance: U256,
    data: &[u8],
) -> Result<DynamicTxPayload<'a>, anyhow::Error> {
    let _data = hex::encode(&data[1..]);

    #[cfg(not(feature = "benchmarking"))]
    log::info!("[+] Decoded data: {_data}");

    let balance = U256::from_dec_str("1000").expect("This should be fine");
    let access = balance >= minimum_balance;

    Ok(subxt::dynamic::tx(
        "Chainlink",
        "callback",
        vec![
            Value::u128(request_id as u128),
            Value::from_bytes([access as u8]),
        ],
    ))
}

async fn get_native_evm_balance(
    _client: &ReqwestClient,
    _chain: Chain,
    _user_address: &Address,
) -> Result<U256, anyhow::Error> {
    Ok(U256::from_dec_str("1000").expect("This should be fine"))
}

async fn get_erc20_balance(
    _client: &ReqwestClient,
    _chain: Chain,
    _user_address: &Address,
    _token_address: &Address,
) -> Result<U256, anyhow::Error> {
    Ok(U256::from_dec_str("10000000000000000").expect("This should be fine"))
}

async fn get_nft(
    _client: &ReqwestClient,
    _chain: Chain,
    _user_address: &Address,
    _token_address: &Address,
    _token_id: U256,
) -> Result<U256, anyhow::Error> {
    Ok(U256::from_dec_str("100").expect("This should be fine"))
}

pub async fn get_evm_balance(
    client: &ReqwestClient,
    token_type: &Option<TokenType<Address, U256>>,
    user_address: &Address,
    chain: Chain,
) -> Result<U256, anyhow::Error> {
    match token_type {
        None => get_native_evm_balance(client, chain, user_address).await,
        Some(TokenType::Fungible {
            address: token_address,
        }) => get_erc20_balance(client, chain, user_address, token_address).await,
        Some(TokenType::NonFungible {
            address: token_address,
            id: token_id,
        }) => get_nft(client, chain, user_address, token_address, *token_id).await,
    }
}

#[cfg(test)]
mod balance_tests {
    use super::{
        get_evm_balance, U256, {Chain, TokenType},
    };
    use crate::address;

    const ZERO_ADDR: &str = "0x0000000000000000000000000000000000000000";

    #[tokio::test]
    pub async fn test_get_eth_balance() {
        let client = reqwest::Client::new();
        let amount = U256::from_dec_str("1000").expect("This should be fine");

        let balance = get_evm_balance(&client, &None, &address!(ZERO_ADDR), Chain::Ethereum)
            .await
            .unwrap();

        assert_eq!(balance, amount);
    }

    #[tokio::test]
    async fn test_get_erc20_balance() {
        let client = reqwest::Client::new();
        let token_type = Some(TokenType::Fungible {
            address: address!("de4e179cc1d3b298216b96893767b9b01a6bc413"),
        });
        let amount = U256::from_dec_str("10000000000000000").expect("This should be fine");

        let balance = get_evm_balance(
            &client,
            &token_type,
            &address!("0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE"),
            Chain::Bsc,
        )
        .await
        .unwrap();

        assert_eq!(balance, amount);
    }

    #[tokio::test]
    pub async fn test_get_nft_balance() {
        let client = reqwest::Client::new();
        let token_type = Some(TokenType::NonFungible {
            address: address!("535053a1cc874c9be92180e599c2529adfbd49f0"),
            id: U256::from_dec_str(
                "44533621599928789006513101770322670729981204560050458486968461274604483117072",
            )
            .expect("This should be fine"),
        });
        let amount = U256::from_dec_str("100").expect("This should be fine");

        let balance = get_evm_balance(
            &client,
            &token_type,
            &address!("0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE"),
            Chain::Polygon,
        )
        .await
        .unwrap();

        assert_eq!(balance, amount);
    }
}

#[cfg(all(test, not(feature = "flood-tests")))]
mod client_tests {
    use super::{process_request, U256};
    use crate::{data::OracleRequestArgs, types::Api};

    use ethock_lib::server::{Entry, ServerType};
    use sp_keyring::AccountKeyring::Alice;
    use std::{thread, time::Duration};
    use subxt::tx::PairSigner;

    const IP: &str = "127.0.0.1";
    const PORT: u16 = 9001;

    #[tokio::test]
    async fn test_process_request() {
        thread::spawn(|| {
            Entry::new(ServerType::WS, &format!("{IP}:{PORT}"), "info").serve_silent()
        });
        thread::sleep(Duration::from_millis(100));

        let api = Api::from_url(&format!("ws://{IP}:{PORT}")).await.unwrap();
        let signer = PairSigner::new(Alice.pair());
        let request = OracleRequestArgs::dummy();
        let minimum_balance = U256::from_dec_str("12345").expect("This should be fine");

        let tx = process_request(request.request_id, minimum_balance, &request.data)
            .await
            .unwrap();

        assert!(api.tx().sign_and_submit_default(&tx, &signer).await.is_ok());
    }
}

#[cfg(all(test, feature = "flood-tests"))]
mod flood_tests {
    use super::balance_tests::{test_get_eth_balance, test_get_nft_balance};
    use std::{
        sync::atomic::{AtomicBool, AtomicUsize, Ordering},
        thread,
        time::Duration,
    };

    const FLOOD_BATCHES: usize = 3;
    const CALLS_PER_BATCH: usize = 200;

    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    static TEST_LOGGER_ENABLED: AtomicBool = AtomicBool::new(false);

    fn init_test_logger() {
        if !TEST_LOGGER_ENABLED.load(Ordering::Relaxed) {
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
                .init();
            TEST_LOGGER_ENABLED.store(true, Ordering::Relaxed);
        }
    }

    async fn flood_test(function: fn(), calls_per_second: usize) -> usize {
        init_test_logger();

        COUNTER.store(0, Ordering::SeqCst);

        for _ in 0..FLOOD_BATCHES {
            for _ in 0..calls_per_second {
                thread::spawn(move || {
                    function();
                    COUNTER.fetch_add(1, Ordering::SeqCst);
                });
            }
            thread::sleep(Duration::from_millis(1000));
        }

        COUNTER.load(Ordering::SeqCst)
    }

    #[tokio::test]
    async fn flood_test_etherscan() {
        let calls = flood_test(test_get_eth_balance, CALLS_PER_BATCH).await;

        log::info!("Calls: {calls}");
    }

    #[tokio::test]
    async fn flood_test_balancy() {
        let calls = flood_test(test_get_nft_balance, CALLS_PER_BATCH).await;

        log::info!("Calls: {calls}");
    }
}
