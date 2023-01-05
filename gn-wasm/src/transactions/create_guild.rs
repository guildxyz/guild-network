use gn_client::data::Guild;
use gn_client::transactions;
use gn_client::{AccountId, Api, PreparedMsgWithParams};

pub async fn create_guild(
    api: Api,
    account_id: &AccountId,
    guild: Guild,
) -> Result<PreparedMsgWithParams, anyhow::Error> {
    let tx_payload = transactions::create_guild(guild)?;
    let prepared = api
        .tx()
        .prepare_unsigned(&tx_payload, &account_id, Default::default())
        .await?;

    Ok(prepared)
}
