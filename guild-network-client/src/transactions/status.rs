use crate::{Hash, TransactionStatus};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TxStatus {
    Ready,
    Broadcast,
    InBlock,
    Finalized,
}

impl TxStatus {
    pub fn reached(self, status: &TransactionStatus) -> (bool, Option<Hash>) {
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

    let status = TransactionStatus::Usurped(Hash::default());
    let (reached, _) = flag.reached(&status);
    assert!(reached);
    let status = TransactionStatus::Retracted(Hash::default());
    let (reached, _) = flag.reached(&status);
    assert!(reached);
    let status = TransactionStatus::FinalityTimeout(Hash::default());
    let (reached, _) = flag.reached(&status);
    assert!(reached);
}
