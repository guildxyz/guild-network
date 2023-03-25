use crate::{SubxtError, TransactionProgress, TransactionStatus, H256};
use futures::StreamExt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TxStatus {
    Ready,
    Broadcast,
    InBlock,
    Finalized,
}

impl TxStatus {
    pub fn reached(self, status: &TransactionStatus) -> (bool, Option<H256>) {
        let mut reached = false;
        let mut tx_hash = None;
        match status {
            TransactionStatus::Future => {}
            TransactionStatus::Ready => {
                if self == Self::Ready {
                    reached = true
                }
            }
            TransactionStatus::Broadcast(_) => {
                if self <= Self::Broadcast {
                    reached = true
                }
            }
            TransactionStatus::InBlock(in_block) => {
                if self <= Self::InBlock {
                    reached = true;
                    tx_hash = Some(in_block.block_hash())
                }
            }
            TransactionStatus::Finalized(in_block) => {
                if self <= Self::Finalized {
                    reached = true;
                    tx_hash = Some(in_block.block_hash());
                }
            }
            _ => reached = true, // these arms represent failed transactions which won't advance
        }
        (reached, tx_hash)
    }
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

#[test]
fn tx_status_reached() {
    let flag = TxStatus::Ready;

    let status = TransactionStatus::Future;
    let (reached, opt) = flag.reached(&status);
    assert!(!reached);
    assert!(opt.is_none());
    let status = TransactionStatus::Ready;
    let (reached, opt) = flag.reached(&status);
    assert!(reached);
    assert!(opt.is_none());

    let flag = TxStatus::Broadcast;

    let (reached, opt) = flag.reached(&status);
    assert!(!reached);
    assert!(opt.is_none());

    let status = TransactionStatus::Broadcast(vec![]);
    let (reached, opt) = flag.reached(&status);
    assert!(reached);
    assert!(opt.is_none());

    let flag = TxStatus::InBlock;

    let status = TransactionStatus::Usurped(H256::default());
    let (reached, _) = flag.reached(&status);
    assert!(reached);
    let status = TransactionStatus::Retracted(H256::default());
    let (reached, _) = flag.reached(&status);
    assert!(reached);
    let status = TransactionStatus::FinalityTimeout(H256::default());
    let (reached, _) = flag.reached(&status);
    assert!(reached);
}
