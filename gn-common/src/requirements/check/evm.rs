use crate::requirements::balance::TokenType;
use crate::requirements::chains::EvmChain;
use crate::requirements::{EvmAddress, U256};
use providers::{evm::general::PROVIDERS, BalanceQuerier, EvmChain as RustyEvmChain};

// only compute this once
const MULTIPLIER: f64 = 1_000_000_000_000_000_000.0; // 10^18

pub async fn get_balance(
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

    let balance = get_balance_with_provider(provider, token_type, user_address).await?;

    let mut result = [0u8; 32];
    result[0..16].copy_from_slice(&balance.to_le_bytes());
    Ok(result)
}

async fn get_balance_with_provider<T>(
    provider: &T,
    token_type: &Option<TokenType<EvmAddress, U256>>,
    user_address: &EvmAddress,
) -> Result<u128, anyhow::Error>
where
    T: BalanceQuerier,
    for<'a> <T as BalanceQuerier>::Address: From<&'a EvmAddress>,
    for<'a> <T as BalanceQuerier>::Id: From<&'a U256>,
    <T as BalanceQuerier>::Balance: std::ops::Mul<f64, Output = f64> + Copy,
{
    let balance = match token_type {
        None => {
            let balances = provider.get_native_balance(&[user_address.into()]).await;
            if let Some(Ok(balance)) = balances.get(0) {
                (*balance * MULTIPLIER) as u128
            } else {
                0
            }
        }
        Some(TokenType::Fungible {
            address: token_address,
        }) => {
            let balances = provider
                .get_fungible_balance(token_address.into(), &[user_address.into()])
                .await;

            if let Some(Ok(balance)) = balances.get(0) {
                (*balance * MULTIPLIER) as u128
            } else {
                0
            }
        }
        Some(TokenType::NonFungible {
            address: token_address,
            id: token_id,
        }) => {
            let balances = provider
                .get_non_fungible_balance(
                    token_address.into(),
                    Some(token_id.into()),
                    &[user_address.into()],
                )
                .await;

            if let Some(Ok(balance)) = balances.get(0) {
                (*balance * 1.0) as u128
            } else {
                0
            }
        }
        Some(TokenType::Special { .. }) => anyhow::bail!("Token type not supported"), // TODO
    };
    Ok(balance)
}

#[cfg(test)]
mod test {
    use super::*;
    use async_trait::async_trait;
    use providers::{Address, U256};

    struct TestProvider;

    #[async_trait]
    impl BalanceQuerier for TestProvider {
        type Address = Address;
        type Id = U256;
        type Balance = f64;
        type Chain = EvmChain;
        type Error = String;
        async fn get_native_balance(
            &self,
            _user_addresses: &[Self::Address],
        ) -> Vec<Result<Self::Balance, Self::Error>> {
            vec![Ok(100.0)]
        }
        async fn get_fungible_balance(
            &self,
            _token_address: Self::Address,
            _user_addresses: &[Self::Address],
        ) -> Vec<Result<Self::Balance, Self::Error>> {
            vec![Ok(10.0)]
        }
        async fn get_non_fungible_balance(
            &self,
            _token_address: Address,
            _token_id: Option<Self::Id>,
            _user_addresses: &[Address],
        ) -> Vec<Result<Self::Balance, Self::Error>> {
            vec![Ok(1.0)]
        }
        async fn get_special_balance(
            &self,
            _token_address: Self::Address,
            _token_id: Option<Self::Id>,
            _user_addresses: &[Self::Address],
        ) -> Vec<Result<Self::Balance, Self::Error>> {
            unimplemented!()
        }
    }

    #[tokio::test]
    pub async fn test_get_eth_balance() {
        // check zero address
        let balance = get_balance_with_provider(&TestProvider, &None, &[12; 20])
            .await
            .unwrap();

        assert_eq!(balance, 100_000_000_000_000_000_000);
    }

    #[tokio::test]
    async fn test_get_erc20_balance() {
        let token_type = Some(TokenType::Fungible { address: [0; 20] });

        let balance = get_balance_with_provider(&TestProvider, &token_type, &[13; 20])
            .await
            .unwrap();

        assert_eq!(balance, 10_000_000_000_000_000_000);
    }

    #[tokio::test]
    pub async fn test_get_nft_balance() {
        let token_type = Some(TokenType::NonFungible {
            address: [2; 20],
            id: [10; 32],
        });

        let balance = get_balance_with_provider(&TestProvider, &token_type, &[14; 20])
            .await
            .unwrap();

        assert_eq!(balance, 1);
    }
}
