use super::status::{track_progress, TxStatus};
use super::Signer;
use crate::{Api, SubxtError, H256};
use subxt::tx::TxPayload;

use std::sync::Arc;

async fn send<T: TxPayload>(
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

pub async fn owned<T: TxPayload>(
    api: Api,
    tx: T,
    signer: Arc<Signer>,
    status: TxStatus,
) -> Result<Option<H256>, SubxtError> {
    send(api, &tx, signer, status).await
}

pub async fn ready<T: TxPayload>(api: Api, tx: &T, signer: Arc<Signer>) -> Result<(), SubxtError> {
    send(api, tx, signer, TxStatus::Ready).await.map(|_| ())
}

pub async fn broadcast<T: TxPayload>(
    api: Api,
    tx: &T,
    signer: Arc<Signer>,
) -> Result<(), SubxtError> {
    send(api, tx, signer, TxStatus::Broadcast).await.map(|_| ())
}

pub async fn in_block<T: TxPayload>(
    api: Api,
    tx: &T,
    signer: Arc<Signer>,
) -> Result<H256, SubxtError> {
    let hash = send(api, tx, signer, TxStatus::InBlock).await?;
    hash.ok_or_else(|| SubxtError::Other("transaction hash is None".into()))
}

pub async fn finalized<T: TxPayload>(
    api: Api,
    tx: &T,
    signer: Arc<Signer>,
) -> Result<H256, SubxtError> {
    let hash = send(api, tx, signer, TxStatus::Finalized).await?;
    hash.ok_or_else(|| SubxtError::Other("transaction hash is None".into()))
}
