use crate::{api, AccountId, Api, Guild, Signer, TransactionProgress, TransactionStatus, TxStatus};
use futures::StreamExt;
use subxt::ext::sp_runtime::MultiAddress;
use subxt::tx::TxPayload;

use std::sync::Arc;

pub fn fund_account(account: &AccountId, amount: u128) -> Result<impl TxPayload, subxt::Error> {
    Ok(api::tx()
        .balances()
        .transfer(MultiAddress::Id(account.clone()), amount))
}

pub fn register_operator() -> Result<impl TxPayload, subxt::Error> {
    Ok(api::tx().chainlink().register_operator())
}

/*
pub async fn create_guild(guild: &Guild)
-> Result<impl TxPayload, subxt::Error> {
{
    Ok(api::tx().guild().create_guild())
}

pub async fn join_guild() {
    todo!();
}
*/

pub async fn send_tx(
    api: Api,
    tx: impl TxPayload,
    signer: Arc<Signer>,
    status: TxStatus,
) -> Result<(), subxt::Error> {
    let mut progress = api
        .tx()
        .sign_and_submit_then_watch_default(&tx, signer.as_ref())
        .await?;

    while let Some(try_event) = progress.next().await {
        let tx_progress_status = try_event?;
        let reached = status.reached(&tx_progress_status);
        if reached {
            log::info!("transaction status {:?} reached", tx_progress_status);
            break;
        }
    }

    Ok(())
}
