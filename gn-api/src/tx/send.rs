use super::status::{track_progress, TxStatus};
use super::{Signer, TxPayloadT};
use crate::{Api, SubxtError, H256};
use futures::future::try_join_all;

use std::ops::Deref;
use std::sync::Arc;

async fn send<T: TxPayloadT>(
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

async fn send_but_dont_watch<T: TxPayloadT>(
    api: Api,
    tx: &T,
    signer: Arc<Signer>,
) -> Result<Option<H256>, SubxtError> {
    Ok(Some(
        api.tx()
            .sign_and_submit_default(tx, signer.as_ref())
            .await?,
    ))
}

pub async fn batch<'a, T, P>(api: Api, payloads: P, signer: Arc<Signer>) -> Result<(), SubxtError>
where
    T: TxPayloadT + 'a,
    P: Iterator<Item = &'a T>,
{
    let account_nonce = api
        .rpc()
        .system_account_next_index(signer.account_id())
        .await?;
    let mut encoded_extrinsics = Vec::<Vec<u8>>::new();
    for (i, payload) in payloads.enumerate() {
        let signed_tx = api.tx().create_signed_with_nonce(
            payload,
            Arc::clone(&signer).as_ref(),
            account_nonce + i as u32,
            Default::default(),
        )?;
        encoded_extrinsics.push(signed_tx.into_encoded());
    }
    let tx_futures = encoded_extrinsics
        .into_iter()
        .map(|ext| {
            api.rpc().deref().request::<H256>(
                "author_submitExtrinsic",
                subxt::rpc::rpc_params![subxt::rpc::types::Bytes::from(ext)],
            )
        })
        .collect::<Vec<_>>();
    try_join_all(tx_futures).await.map(|_| ())
}

pub async fn owned<T: TxPayloadT>(
    api: Api,
    tx: T,
    signer: Arc<Signer>,
    status: TxStatus,
) -> Result<Option<H256>, SubxtError> {
    send(api, &tx, signer, status).await
}

pub async fn owned_but_dont_watch<T: TxPayloadT>(
    api: Api,
    tx: T,
    signer: Arc<Signer>,
) -> Result<Option<H256>, SubxtError> {
    send_but_dont_watch(api, &tx, signer).await
}

pub async fn ready<T: TxPayloadT>(api: Api, tx: &T, signer: Arc<Signer>) -> Result<(), SubxtError> {
    send(api, tx, signer, TxStatus::Ready).await.map(|_| ())
}

pub async fn broadcast<T: TxPayloadT>(
    api: Api,
    tx: &T,
    signer: Arc<Signer>,
) -> Result<(), SubxtError> {
    send(api, tx, signer, TxStatus::Broadcast).await.map(|_| ())
}

pub async fn in_block<T: TxPayloadT>(
    api: Api,
    tx: &T,
    signer: Arc<Signer>,
) -> Result<H256, SubxtError> {
    let hash = send(api, tx, signer, TxStatus::InBlock).await?;
    hash.ok_or_else(|| SubxtError::Other("transaction hash is None".into()))
}

pub async fn finalized<T: TxPayloadT>(
    api: Api,
    tx: &T,
    signer: Arc<Signer>,
) -> Result<H256, SubxtError> {
    let hash = send(api, tx, signer, TxStatus::Finalized).await?;
    hash.ok_or_else(|| SubxtError::Other("transaction hash is None".into()))
}
