use super::Requirement;
use crate::identities::Identity;
use reqwest::Client as ReqwestClient;

mod evm;

#[derive(Clone, Copy)]
pub enum Chain {
    Ethereum = 1,
    Bsc = 56,
    Gnosis = 100,
    Polygon = 137,
}

impl Requirement {
    pub async fn check(
        &self,
        client: &ReqwestClient,
        user_identity: &Identity,
    ) -> Result<(), anyhow::Error> {
        let is_valid = match (self, user_identity) {
            (Self::Free, _) => true,
            (Self::EthereumBalance(req_balance), Identity::EvmChain(user_address)) => {
                let balance = evm::get_balance(
                    client,
                    &req_balance.token_type,
                    user_address,
                    Chain::Ethereum,
                )
                .await?;
                req_balance.relation.assert(&balance, &req_balance.amount)
            }
            (Self::BscBalance(req_balance), Identity::EvmChain(user_address)) => {
                let balance =
                    evm::get_balance(client, &req_balance.token_type, user_address, Chain::Bsc)
                        .await?;
                req_balance.relation.assert(&balance, &req_balance.amount)
            }
            (Self::GnosisBalance(req_balance), Identity::EvmChain(user_address)) => {
                let balance =
                    evm::get_balance(client, &req_balance.token_type, user_address, Chain::Gnosis)
                        .await?;
                req_balance.relation.assert(&balance, &req_balance.amount)
            }
            (Self::PolygonBalance(req_balance), Identity::EvmChain(user_address)) => {
                let balance = evm::get_balance(
                    client,
                    &req_balance.token_type,
                    user_address,
                    Chain::Polygon,
                )
                .await?;
                req_balance.relation.assert(&balance, &req_balance.amount)
            }
            (Self::EvmAllowlist(allowlist), Identity::EvmChain(user_address)) => {
                allowlist.is_member(user_address)
            }
            _ => false,
        };

        anyhow::ensure!(is_valid, "requirement check failed");
        Ok(())
    }
}
