mod status;
pub use sp_core::crypto::Pair as PairT;
pub use sp_core::sr25519::Pair as Keypair;
pub use status::TxStatus;
pub use subxt::tx::Signer as SignerT;

pub type Signer = subxt::tx::PairSigner<ClientConfig, Keypair>;

use crate::{
    cast, runtime, AccountId, Api, ClientConfig, MultiAddress, SubxtError, TransactionProgress,
    H256,
};
use futures::StreamExt;
use gn_common::filter::{Guild as GuildFilter, Logic as FilterLogic};
use gn_common::identity::{Identity, IdentityWithAuth};
use gn_common::merkle::Proof as MerkleProof;
use gn_common::{GuildName, RoleName};
use gn_engine::RequirementsWithLogic;
use subxt::dynamic::Value;
use subxt::tx::{DynamicTxPayload, TxPayload};

use std::sync::Arc;

pub fn sudo<'a, 'b>(call: DynamicTxPayload<'a>) -> DynamicTxPayload<'b> {
    subxt::dynamic::tx("Sudo", "sudo", vec![("call", call.into_value())])
}

pub fn fund_account(account: &AccountId, amount: u128) -> impl TxPayload {
    runtime::tx()
        .balances()
        .transfer(MultiAddress::Id(account.clone()), amount)
}

pub fn register_operator<'a>(operator: &AccountId) -> DynamicTxPayload<'a> {
    subxt::dynamic::tx(
        "Oracle",
        "register_operator",
        vec![("operator", Value::from_bytes(operator))],
    )
}

pub fn oracle_callback(request_id: u64, data: Vec<u8>) -> impl TxPayload {
    runtime::tx().oracle().callback(request_id, data)
}

pub fn create_guild(guild_name: GuildName, metadata: Vec<u8>) -> impl TxPayload {
    runtime::tx().guild().create_guild(guild_name, metadata)
}

pub fn create_free_role(guild_name: GuildName, role_name: RoleName) -> impl TxPayload {
    runtime::tx()
        .guild()
        .create_free_role(guild_name, role_name)
}

pub fn create_role_with_allowlist(
    guild_name: GuildName,
    role_name: RoleName,
    allowlist: Vec<Identity>,
    filter_logic: FilterLogic,
    requirements: Option<RequirementsWithLogic>,
) -> Result<impl TxPayload, SubxtError> {
    let serialized_requirements = requirements
        .map(RequirementsWithLogic::into_serialized_tuple)
        .transpose()
        .map_err(|e| SubxtError::Other(e.to_string()))?;
    Ok(runtime::tx().guild().create_role_with_allowlist(
        guild_name,
        role_name,
        cast::id_vec::to_runtime(allowlist),
        cast::filter_logic::to_runtime(filter_logic),
        serialized_requirements,
    ))
}

pub fn create_child_role(
    guild_name: GuildName,
    role_name: RoleName,
    filter: GuildFilter,
    filter_logic: FilterLogic,
    requirements: Option<RequirementsWithLogic>,
) -> Result<impl TxPayload, SubxtError> {
    let serialized_requirements = requirements
        .map(RequirementsWithLogic::into_serialized_tuple)
        .transpose()
        .map_err(|e| SubxtError::Other(e.to_string()))?;
    Ok(runtime::tx().guild().create_child_role(
        guild_name,
        role_name,
        cast::guild_filter::to_runtime(filter),
        cast::filter_logic::to_runtime(filter_logic),
        serialized_requirements,
    ))
}

pub fn create_unfiltered_role(
    guild_name: GuildName,
    role_name: RoleName,
    requirements: RequirementsWithLogic,
) -> Result<impl TxPayload, SubxtError> {
    let serialized_requirements = requirements
        .into_serialized_tuple()
        .map_err(|e| SubxtError::Other(e.to_string()))?;
    Ok(runtime::tx()
        .guild()
        .create_unfiltered_role(guild_name, role_name, serialized_requirements))
}

pub fn register(identity_with_auth: IdentityWithAuth, index: u8) -> impl TxPayload {
    runtime::tx()
        .guild()
        .register(cast::id_with_auth::to_runtime(identity_with_auth), index)
}

pub fn join(
    guild_name: GuildName,
    role_name: RoleName,
    proof: Option<MerkleProof>,
) -> impl TxPayload {
    runtime::tx()
        .guild()
        .join(guild_name, role_name, proof.map(cast::proof::to_runtime))
}

pub fn leave(guild_name: GuildName, role_name: RoleName) -> impl TxPayload {
    runtime::tx().guild().leave(guild_name, role_name)
}

pub async fn send_owned_tx<T: TxPayload>(
    api: Api,
    tx: T,
    signer: Arc<Signer>,
    status: TxStatus,
) -> Result<Option<H256>, SubxtError> {
    send_tx(api, &tx, signer, status).await
}

pub async fn send_tx<T: TxPayload>(
    api: Api,
    tx: &T,
    signer: Arc<Signer>,
    status: TxStatus,
) -> Result<Option<H256>, SubxtError> {
    let mut progress = api
        .tx()
        .sign_and_submit_then_watch_default(tx, signer.as_ref())
        .await?;

    track_progress(&mut progress, status).await
}

pub async fn track_progress(
    progress: &mut TransactionProgress,
    status: TxStatus,
) -> Result<Option<H256>, SubxtError> {
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
) -> Result<H256, SubxtError> {
    let hash = send_tx(api, tx, signer, TxStatus::InBlock).await?;
    hash.ok_or_else(|| SubxtError::Other("transaction hash is None".into()))
}

pub async fn send_tx_finalized<T: TxPayload>(
    api: Api,
    tx: &T,
    signer: Arc<Signer>,
) -> Result<H256, SubxtError> {
    let hash = send_tx(api, tx, signer, TxStatus::Finalized).await?;
    hash.ok_or_else(|| SubxtError::Other("transaction hash is None".into()))
}
