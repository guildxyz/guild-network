use crate::{api, AccountId, Api, Guild, Signer, TxStatus};
use futures::StreamExt;
use guild_network_gate::identities::{Identity, IdentityAuth};
use subxt::ext::sp_runtime::MultiAddress;
use subxt::tx::TxPayload;

use std::sync::Arc;

pub fn fund_account(account: &AccountId, amount: u128) -> impl TxPayload {
    api::tx()
        .balances()
        .transfer(MultiAddress::Id(account.clone()), amount)
}

pub fn register_operator() -> impl TxPayload {
    api::tx().chainlink().register_operator()
}

pub fn oracle_callback(request_id: u64, data: Vec<u8>) -> impl TxPayload {
    api::tx().chainlink().callback(request_id, data)
}

pub async fn create_guild(guild: Guild) -> impl TxPayload {
    // TODO serialize requirements
    //let roles = guild
    //    .roles
    //    .into_iter()
    //    .map(|role| (role.name, role.requirements))
    //    .collect();
    let roles = vec![([0; 32], vec![11])];
    api::tx()
        .guild()
        .create_guild(guild.name, guild.metadata, roles)
}

pub async fn join_guild(
    guild_name: [u8; 32],
    role_name: [u8; 32],
    _identities: Vec<Identity>,
    _auth: Vec<IdentityAuth>,
) -> impl TxPayload {
    // TODO serialize requirements
    api::tx()
        .guild()
        .join_guild(guild_name, role_name, vec![0], vec![1])
}

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
