use crate::{runtime, AccountId, Api, Guild, Signer, TxHash, TxStatus};
use futures::StreamExt;
use guild_network_gate::identities::{Identity, IdentityAuth};
use subxt::ext::sp_runtime::MultiAddress;
use subxt::tx::TxPayload;

use std::sync::Arc;

use runtime::runtime_types::pallet_guild::pallet::Call as GuildCall;

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

pub fn oracle_init_request(
    data_version: u64,
    data: Vec<u8>,
    fee: u128,
    callback: GuildCall,
) -> impl TxPayload {
    runtime::tx()
        .chainlink()
        .initiate_request(data_version, data, fee, callback)
}

pub fn create_guild(guild: Guild) -> impl TxPayload {
    let roles = guild
        .roles
        .into_iter()
        .map(|role| (role.name, vec![12])) // TODO serialize requirements
        .collect();
    runtime::tx()
        .guild()
        .create_guild(guild.name, guild.metadata, roles)
}

pub fn join_guild(
    guild_name: [u8; 32],
    role_name: [u8; 32],
    _identities: Vec<Identity>,
    _auth: Vec<IdentityAuth>,
) -> impl TxPayload {
    // TODO serialize identities
    runtime::tx()
        .guild()
        .join_guild(guild_name, role_name, vec![0], vec![1])
}

pub async fn send_tx(
    api: Api,
    tx: impl TxPayload,
    signer: Arc<Signer>,
    status: TxStatus,
) -> Result<Option<TxHash>, subxt::Error> {
    let mut progress = api
        .tx()
        .sign_and_submit_then_watch_default(&tx, signer.as_ref())
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

pub async fn send_tx_ready(
    api: Api,
    tx: impl TxPayload,
    signer: Arc<Signer>,
) -> Result<(), subxt::Error> {
    send_tx(api, tx, signer, TxStatus::Ready).await.map(|_| ())
}

pub async fn send_tx_broadcast(
    api: Api,
    tx: impl TxPayload,
    signer: Arc<Signer>,
) -> Result<(), subxt::Error> {
    send_tx(api, tx, signer, TxStatus::Broadcast)
        .await
        .map(|_| ())
}

pub async fn send_tx_in_block(
    api: Api,
    tx: impl TxPayload,
    signer: Arc<Signer>,
) -> Result<TxHash, subxt::Error> {
    let hash = send_tx(api, tx, signer, TxStatus::InBlock).await?;
    hash.ok_or_else(|| subxt::Error::Other("transaction hash is None".into()))
}

pub async fn send_tx_finalized(
    api: Api,
    tx: impl TxPayload,
    signer: Arc<Signer>,
) -> Result<TxHash, subxt::Error> {
    let hash = send_tx(api, tx, signer, TxStatus::Finalized).await?;
    hash.ok_or_else(|| subxt::Error::Other("transaction hash is None".into()))
}
