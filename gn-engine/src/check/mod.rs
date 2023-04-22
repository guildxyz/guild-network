use super::Requirement;
use gn_common::Identity;

mod evm;

impl Requirement {
    pub async fn check(&self, identity: &Identity) -> Result<bool, anyhow::Error> {
        match self {
            Self::EvmBalance(req_balance) => {
                let address = identity[12..]
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("invalid identity"))?;
                let balance =
                    evm::get_balance(&req_balance.token_type, address, req_balance.chain).await?;
                Ok(req_balance.relation.assert(&balance))
            }
        }
    }
}
