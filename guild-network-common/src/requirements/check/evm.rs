use super::*;
use crate::requirements::balance::TokenType;
use crate::{EvmAddress, U256};
use providers::{evm::general::PROVIDERS, BalanceQuerier, EvmChain};

pub async fn get_balance(
    _client: &ReqwestClient,
    token_type: &Option<TokenType<EvmAddress, U256>>,
    user_address: &EvmAddress,
    chain: EvmChain,
) -> Result<U256, anyhow::Error> {
    let Some(provider) = PROVIDERS.get(&(chain as u8)) else {
        anyhow::bail!("Chain not supported")
    };

    let results = match token_type {
        None => provider.get_native_balance(&[user_address.into()]).await,
        Some(TokenType::Fungible {
            address: token_address,
        }) => {
            provider
                .get_fungible_balance(token_address.into(), &[user_address.into()])
                .await
        }
        Some(TokenType::NonFungible {
            address: token_address,
            id: token_id,
        }) => {
            provider
                .get_non_fungible_balance(
                    token_address.into(),
                    Some(token_id.into()),
                    &[user_address.into()],
                )
                .await
        }
        Some(TokenType::Special { .. }) => todo!(),
    };

    let multiplier = 10_u128.pow(18);
    let balance = *(results[0].as_ref().unwrap_or(&0.0)) as u128 * multiplier;

    let mut result = [0u8; 32];
    result[0..16].copy_from_slice(&balance.to_le_bytes());
    Ok(result)
}

/*
#[cfg(test)]
mod balance_tests {
    use super::{
        get_balance, U256, {EvmChain, TokenType},
    };
    use crate::address;

    const ZERO_ADDR: &str = "0x0000000000000000000000000000000000000000";

    #[tokio::test]
    pub async fn test_get_eth_balance() {
        let client = reqwest::Client::new();
        let amount = U256::from_dec_str("1000").expect("This should be fine");

        let balance = get_balance(&client, &None, &address!(ZERO_ADDR), EvmChain::Ethereum)
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
            EvmChain::Bsc,
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
            EvmChain::Polygon,
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
*/
