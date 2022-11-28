use crate::{cbor_serialize, data::Guild, runtime, AccountId, Api, Hash, Signer, TxStatus};
use futures::StreamExt;
use guild_network_common::{GuildName, RoleName};
use guild_network_gate::identities::{Identity, IdentityAuth};
use subxt::ext::sp_runtime::MultiAddress;
use subxt::tx::TxPayload;

use std::sync::Arc;

pub fn fund_account(account: &AccountId, amount: u128) -> impl TxPayload {
    runtime::tx()
        .balances()
        .transfer(MultiAddress::Id(account.clone()), amount)
}

pub fn register_operator() -> impl TxPayload {
    runtime::tx().chainlink().register_operator()
}

pub fn oracle_callback(request_id: u64, data: Vec<u8>) -> impl TxPayload {
    runtime::tx().chainlink().callback(request_id, data)
}

pub fn create_guild(guild: Guild) -> Result<impl TxPayload, serde_cbor::Error> {
    let mut roles = Vec::new();
    for role in guild.roles.into_iter() {
        let ser_requirements = cbor_serialize(&role.requirements)?;
        roles.push((role.name, ser_requirements));
    }

    Ok(runtime::tx()
        .guild()
        .create_guild(guild.name, guild.metadata, roles))
}

pub fn join_guild(
    guild_name: GuildName,
    role_name: RoleName,
    identities: Vec<Identity>,
    auth: Vec<IdentityAuth>,
) -> Result<impl TxPayload, serde_cbor::Error> {
    let ser_identities = cbor_serialize(&identities)?;
    let ser_auth = cbor_serialize(&auth)?;
    Ok(runtime::tx()
        .guild()
        .join_guild(guild_name, role_name, ser_identities, ser_auth))
}

pub async fn send_owned_tx<T: TxPayload>(
    api: Api,
    tx: T,
    signer: Arc<Signer>,
    status: TxStatus,
) -> Result<Option<Hash>, subxt::Error> {
    send_tx(api, &tx, signer, status).await
}

pub async fn send_tx<T: TxPayload>(
    api: Api,
    tx: &T,
    signer: Arc<Signer>,
    status: TxStatus,
) -> Result<Option<Hash>, subxt::Error> {
    let mut progress = api
        .tx()
        .sign_and_submit_then_watch_default(tx, signer.as_ref())
        .await?;

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
) -> Result<(), subxt::Error> {
    send_tx(api, tx, signer, TxStatus::Ready).await.map(|_| ())
}

pub async fn send_tx_broadcast<T: TxPayload>(
    api: Api,
    tx: &T,
    signer: Arc<Signer>,
) -> Result<(), subxt::Error> {
    send_tx(api, tx, signer, TxStatus::Broadcast)
        .await
        .map(|_| ())
}

pub async fn send_tx_in_block<T: TxPayload>(
    api: Api,
    tx: &T,
    signer: Arc<Signer>,
) -> Result<Hash, subxt::Error> {
    let hash = send_tx(api, tx, signer, TxStatus::InBlock).await?;
    hash.ok_or_else(|| subxt::Error::Other("transaction hash is None".into()))
}

pub async fn send_tx_finalized<T: TxPayload>(
    api: Api,
    tx: &T,
    signer: Arc<Signer>,
) -> Result<Hash, subxt::Error> {
    let hash = send_tx(api, tx, signer, TxStatus::Finalized).await?;
    hash.ok_or_else(|| subxt::Error::Other("transaction hash is None".into()))
}
