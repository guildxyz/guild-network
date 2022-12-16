use crate::requirements::balance::TokenType;
use crate::requirements::chains::EvmChain;
use crate::{EvmAddress, U256};
use providers::{evm::general::PROVIDERS, BalanceQuerier, EvmChain as RustyEvmChain};
use reqwest::Client as ReqwestClient;

// only compute this once
const MULTIPLIER: u128 = 1_000_000_000_000_000_000; // 10^18

pub async fn get_balance(
    _client: &ReqwestClient,
    token_type: &Option<TokenType<EvmAddress, U256>>,
    user_address: &EvmAddress,
    chain: EvmChain,
) -> Result<U256, anyhow::Error> {
    // TODO I don't know how to de-duplicate `EvmChain` yet.
    // The main problem is that we need special traits to be derived
    // for these types (like Encode, Decode) which makes it less
    // suitable to include in a generic library like rusty-gate.
    // We'll have to think about this, because the same pertains
    // to other duplicate types (like TokenType)
    let chain_id = match chain {
        EvmChain::Ethereum => RustyEvmChain::Ethereum as u8,
        EvmChain::Bsc => RustyEvmChain::Bsc as u8,
        EvmChain::Gnosis => RustyEvmChain::Gnosis as u8,
        EvmChain::Polygon => RustyEvmChain::Polygon as u8,
    };
    let Some(provider) = PROVIDERS.get(&chain_id) else {
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

    // TODO because here we get vec of results, we need to index into it which
    // is quite inconvenient and it "might" panic if the lenght is 0. So use
    // `get` instead, but that returns an Option so we need a double unwrap_or
    //
    // we wouldn't need any of this if the get_balance would return and U256
    let balance = if let Some(Ok(balance)) = results.get(0) {
        *balance as u128 * MULTIPLIER
    } else {
        0
    };

    let mut result = [0u8; 32];
    result[0..16].copy_from_slice(&balance.to_le_bytes());
    Ok(result)
}

#[cfg(test)]
mod test {
    use super::*;

    // 0xde4e179cc1d3b298216b96893767b9b01a6bc413
    const TOKEN_ADDRESS: EvmAddress = [
        222, 78, 23, 156, 193, 211, 178, 152, 33, 107, 150, 137, 55, 103, 185, 176, 26, 107, 196,
        19,
    ];
    // 0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE
    const USER_ADDRESS: EvmAddress = [
        228, 56, 120, 206, 120, 147, 79, 232, 0, 119, 72, 255, 72, 31, 3, 184, 238, 59, 151, 222,
    ];
    // 0x535053a1cc874c9be92180e599c2529adfbd49f0
    const NFT_ADDRESS: EvmAddress = [
        83, 80, 83, 161, 204, 135, 76, 155, 233, 33, 128, 229, 153, 194, 82, 154, 223, 189, 73, 240,
    ];
    // 44533621599928789006513101770322670729981204560050458486968461274604483117072
    const ID: U256 = [
        16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 220, 221, 62, 82, 235, 65, 53, 128, 171, 116, 103,
        193, 128, 219, 39, 130, 247, 34, 117, 98,
    ];
    // 10_000_000_000_000_000
    const BALANCE_E16: U256 = [
        0, 0, 193, 111, 242, 134, 35, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0,
    ];
    // 1000
    const BALANCE_E3: U256 = [
        232, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ];
    // 100
    const BALANCE_E2: U256 = [
        100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ];

    #[tokio::test]
    pub async fn test_get_eth_balance() {
        let client = reqwest::Client::new();

        // check zero address
        let balance = get_balance(&client, &None, &[0; 20], EvmChain::Ethereum)
            .await
            .unwrap();

        assert_eq!(balance, BALANCE_E3);
    }

    #[tokio::test]
    async fn test_get_erc20_balance() {
        let client = reqwest::Client::new();
        let token_type = Some(TokenType::Fungible {
            address: TOKEN_ADDRESS,
        });

        let balance = get_balance(&client, &token_type, &USER_ADDRESS, EvmChain::Bsc)
            .await
            .unwrap();

        assert_eq!(balance, BALANCE_E16);
    }

    #[tokio::test]
    pub async fn test_get_nft_balance() {
        let client = reqwest::Client::new();
        let token_type = Some(TokenType::NonFungible {
            address: NFT_ADDRESS,
            id: ID,
        });

        let balance = get_balance(&client, &token_type, &USER_ADDRESS, EvmChain::Polygon)
            .await
            .unwrap();

        assert_eq!(balance, BALANCE_E2);
    }
}
