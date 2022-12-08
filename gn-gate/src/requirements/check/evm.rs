use super::super::balance::TokenType;
use super::*;
use ethereum_types::{Address, U256};

async fn get_native_balance(
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

pub async fn get_balance(
    client: &ReqwestClient,
    token_type: &Option<TokenType<Address, U256>>,
    user_address: &Address,
    chain: Chain,
) -> Result<U256, anyhow::Error> {
    match token_type {
        None => get_native_balance(client, chain, user_address).await,
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
        get_balance, U256, {Chain, TokenType},
    };
    use crate::address;

    const ZERO_ADDR: &str = "0x0000000000000000000000000000000000000000";

    #[tokio::test]
    pub async fn test_get_eth_balance() {
        let client = reqwest::Client::new();
        let amount = U256::from_dec_str("1000").expect("This should be fine");

        let balance = get_balance(&client, &None, &address!(ZERO_ADDR), Chain::Ethereum)
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

        let balance = get_balance(
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

        let balance = get_balance(
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

#[cfg(all(test, feature = "flood-tests"))]
mod flood_tests {
    use super::balance_tests::{test_get_eth_balance, test_get_nft_balance};
    use std::{
        sync::atomic::{AtomicUsize, Ordering},
        thread,
        time::Duration,
    };

    const FLOOD_BATCHES: usize = 3;
    const CALLS_PER_BATCH: usize = 200;
    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    async fn flood_test(function: fn(), calls_per_second: usize) -> usize {
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

        println!("Calls: {calls}");
    }

    #[tokio::test]
    async fn flood_test_balancy() {
        let calls = flood_test(test_get_nft_balance, CALLS_PER_BATCH).await;

        println!("Calls: {calls}");
    }
}