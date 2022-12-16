use gn_client::transactions;
use gn_client::{AccountId, Api, PreparedMsgWithParams};
use gn_common::pad::pad_to_32_bytes;

pub async fn join_guild(
    api: Api,
    account_id: &AccountId,
    guild: String,
    role: String,
) -> Result<PreparedMsgWithParams, anyhow::Error> {
    anyhow::ensure!(
        guild.len() <= 32 && role.len() <= 32,
        "too long input length"
    );
    let tx_payload = transactions::join_guild(pad_to_32_bytes(&guild), pad_to_32_bytes(&role));
    let prepared = api
        .tx()
        .prepare_unsigned(&tx_payload, &account_id, Default::default())
        .await?;

    Ok(prepared)
}
