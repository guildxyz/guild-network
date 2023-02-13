use super::Requirement;
use gn_common::identity::Identity;

mod evm;

impl Requirement {
    pub async fn check(&self, identity: &Identity) -> Result<bool, anyhow::Error> {
        match self {
            Self::EvmBalance(req_balance) => {
                if let Identity::Address20(address) = identity {
                    let balance =
                        evm::get_balance(&req_balance.token_type, address, req_balance.chain)
                            .await?;
                    Ok(req_balance.relation.assert(&balance))
                } else {
                    Err(anyhow::anyhow!("invalid evm identity"))
                }
            }
        }
    }
}
