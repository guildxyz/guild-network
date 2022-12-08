use super::Requirement;
use crate::identities::IdentityMap;
use reqwest::Client as ReqwestClient;

mod evm;

impl Requirement {
    pub async fn check(
        &self,
        client: &ReqwestClient,
        identity_map: &IdentityMap,
    ) -> Result<bool, anyhow::Error> {
        match self {
            Self::Free => Ok(true),
            Self::EvmBalance(req_balance) => {
                if let Some(address) = identity_map.evm_address() {
                    let balance = evm::get_balance(
                        client,
                        &req_balance.token_type,
                        address,
                        req_balance.chain,
                    )
                    .await?;
                    Ok(req_balance.relation.assert(&balance))
                } else {
                    Err(anyhow::anyhow!("missing evm identity"))
                }
            }
            Self::EvmAllowlist(allowlist) => {
                if let Some(address) = identity_map.evm_address() {
                    Ok(allowlist.is_member(address))
                } else {
                    Err(anyhow::anyhow!("missing evm identity"))
                }
            }
        }
    }
}
