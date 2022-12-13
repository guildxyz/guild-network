use crate::requirements::balance::TokenType;
use crate::requirements::chains::EvmChain;
use crate::{EvmAddress, U256};
use reqwest::Client as ReqwestClient;

async fn get_native_balance(
    _client: &ReqwestClient,
    _chain: EvmChain,
    _user_address: &EvmAddress,
) -> Result<U256, anyhow::Error> {
    let mut result = [0u8; 32];
    result[0..8].copy_from_slice(&1000u64.to_le_bytes());
    Ok(result)
}

async fn get_erc20_balance(
    _client: &ReqwestClient,
    _chain: EvmChain,
    _user_address: &EvmAddress,
    _token_address: &EvmAddress,
) -> Result<U256, anyhow::Error> {
    let mut result = [0u8; 32];
    result[0..16].copy_from_slice(&1_000_000_000_000_000u128.to_le_bytes());
    Ok(result)
}

async fn get_nft(
    _client: &ReqwestClient,
    _chain: EvmChain,
    _user_address: &EvmAddress,
    _token_address: &EvmAddress,
    _token_id: U256,
) -> Result<U256, anyhow::Error> {
    let mut result = [0u8; 32];
    result[0] = 1;
    Ok(result)
}

pub async fn get_balance(
    client: &ReqwestClient,
    token_type: &Option<TokenType<EvmAddress, U256>>,
    user_address: &EvmAddress,
    chain: EvmChain,
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
        Some(TokenType::Special { .. }) => todo!(),
    }
}
