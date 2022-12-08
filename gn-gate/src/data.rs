use crate::types::*;
use codec::Decode;
use ethers::types::{Address, U256};
use serde::Deserialize;

#[derive(Decode, Debug)]
pub struct OracleRequestArgs {
    pub operator_id: AccountId,
    pub request_id: RequestIdentifier,
    _user_id: AccountId,
    _data_version: DataVersion,
    pub data: SpVec<u8>,
    _callback: SpVec<u8>,
    _fee: BalanceOf,
}

#[derive(Deserialize, Debug)]
pub struct EtherscanResponse {
    pub result: U256,
}

#[derive(Deserialize, Debug)]
pub struct Erc20 {
    pub address: Address,
    pub amount: U256,
}

#[serde(rename_all = "camelCase")]
#[derive(Deserialize, Debug)]
pub struct Erc721 {
    pub address: Address,
    pub token_id: U256,
}

#[serde(rename_all = "PascalCase")]
#[derive(Deserialize, Debug)]
pub struct Erc1155 {
    #[serde(rename = "address")]
    pub addr: Address,
    pub token_id: U256,
    pub amount: U256,
}

#[derive(Deserialize, Debug)]
pub struct BalancyResponse {
    pub erc20: Vec<Erc20>,
    pub erc721: Vec<Erc721>,
    pub erc1155: Vec<Erc1155>,
}

impl OracleRequestArgs {
    #[cfg(any(test, feature = "benchmarking"))]
    pub fn dummy() -> Self {
        Self {
            operator_id: AccountId::new([0; 32]),
            request_id: 0,
            _user_id: AccountId::new([0; 32]),
            _data_version: 0,
            data: vec![
                80, 32, 204, 84, 199, 235, 197, 244, 59, 116, 134, 109, 131, 155, 75, 213, 192, 27,
                178, 53, 3,
            ],
            _callback: vec![],
            _fee: 100_000_000,
        }
    }
}
