mod status;
pub use status::TxStatus;

use crate::{
    cbor_serialize, data::Guild, runtime, AccountId, Api, Hash, RequestData,
    RuntimeIdentityWithAuth, Signer, SubxtError, TransactionProgress,
};
use futures::StreamExt;
use gn_common::{GuildName, RoleName};
use subxt::ext::sp_runtime::MultiAddress;
use subxt::tx::TxPayload;

use std::sync::Arc;

pub fn fund_account(account: &AccountId, amount: u128) -> impl TxPayload {
    runtime::tx()
        .balances()
        .transfer(MultiAddress::Id(account.clone()), amount)
}

pub fn register_operator() -> impl TxPayload {
    runtime::tx().oracle().register_operator()
}

pub fn oracle_callback(request_id: u64, data: Vec<u8>) -> impl TxPayload {
    runtime::tx().oracle().callback(request_id, data)
}

pub fn create_guild(guild: Guild) -> Result<impl TxPayload, serde_cbor::Error> {
    let mut roles = Vec::new();
    for role in guild.roles.into_iter() {
        let ser_requirements = cbor_serialize(&role.reqs)?;
        roles.push((role.name, ser_requirements));
    }

    Ok(runtime::tx()
        .guild()
        .create_guild(guild.name, guild.metadata, roles))
}

pub fn register(identities: Vec<RuntimeIdentityWithAuth>) -> impl TxPayload {
    runtime::tx()
        .guild()
        .register(RequestData::Register(identities))
}

pub fn join_guild(guild_name: GuildName, role_name: RoleName) -> impl TxPayload {
    runtime::tx().guild().join_guild(RequestData::Join {
        guild: guild_name,
        role: role_name,
    })
}

pub async fn send_owned_tx<T: TxPayload>(
    api: Api,
    tx: T,
    signer: Arc<Signer>,
    status: TxStatus,
) -> Result<Option<Hash>, SubxtError> {
    send_tx(api, &tx, signer, status).await
}

pub async fn send_tx<T: TxPayload>(
    api: Api,
    tx: &T,
    signer: Arc<Signer>,
    status: TxStatus,
) -> Result<Option<Hash>, SubxtError> {
    let mut progress = api
        .tx()
        .sign_and_submit_then_watch_default(tx, signer.as_ref())
        .await?;

    track_progress(&mut progress, status).await
}

pub async fn track_progress(
    progress: &mut TransactionProgress,
    status: TxStatus,
) -> Result<Option<Hash>, SubxtError> {
    while let Some(try_event) = progress.next().await {
        let tx_progress_status = try_event?;
        let (reached, tx_hash) = status.reached(&tx_progress_status);
        if reached {
            log::info!(
                "transaction status {:?} reached, hash: {:?}",
                status,
                tx_hash
            );
            return Ok(tx_hash);
        }
    }
    Ok(None)
}

pub async fn send_tx_ready<T: TxPayload>(
    api: Api,
    tx: &T,
    signer: Arc<Signer>,
) -> Result<(), SubxtError> {
    send_tx(api, tx, signer, TxStatus::Ready).await.map(|_| ())
}

pub async fn send_tx_broadcast<T: TxPayload>(
    api: Api,
    tx: &T,
    signer: Arc<Signer>,
) -> Result<(), SubxtError> {
    send_tx(api, tx, signer, TxStatus::Broadcast)
        .await
        .map(|_| ())
}

pub async fn send_tx_in_block<T: TxPayload>(
    api: Api,
    tx: &T,
    signer: Arc<Signer>,
) -> Result<Hash, SubxtError> {
    let hash = send_tx(api, tx, signer, TxStatus::InBlock).await?;
    hash.ok_or_else(|| SubxtError::Other("transaction hash is None".into()))
}

pub async fn send_tx_finalized<T: TxPayload>(
    api: Api,
    tx: &T,
    signer: Arc<Signer>,
) -> Result<Hash, SubxtError> {
    let hash = send_tx(api, tx, signer, TxStatus::Finalized).await?;
    hash.ok_or_else(|| SubxtError::Other("transaction hash is None".into()))
}
