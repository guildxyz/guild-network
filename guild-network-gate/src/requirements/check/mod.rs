use super::Requirement;
use crate::identities::Identity;
use reqwest::Client as ReqwestClient;

mod evm;

impl Requirement {
    pub async fn check(
        &self,
        client: &ReqwestClient,
        user_identity: &Identity,
    ) -> Result<(), anyhow::Error> {
        let is_valid = match (self, user_identity) {
            (Self::Free, _) => true,
            (Self::EvmBalance(req_balance), Identity::EvmChain(user_address)) => {
                let balance = evm::get_balance(
                    client,
                    &req_balance.token_type,
                    user_address,
                    req_balance.chain,
                )
                .await?;
                req_balance.relation.assert(&balance)
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
