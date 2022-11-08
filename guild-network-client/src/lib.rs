use guild_network_gate::requirements::Requirement;
use subxt::{
    events::{EventSubscription, FilterEvents},
    ext::sp_runtime::{generic::Header, traits::BlakeTwo256, AccountId32},
    rpc::Subscription,
    tx::{PairSigner, TxProgress, TxStatus as SubTxStatus},
    OnlineClient,
};

// re-exports
pub use sp_keyring::sr25519::sr25519::Pair as Keypair;
pub use subxt::ext::sp_core::H256 as TxHash;
pub use subxt::tx::Signer as TxSignerTrait;
pub use subxt::PolkadotConfig as ClientConfig;

#[subxt::subxt(runtime_metadata_path = "./artifacts/metadata.scale")]
pub mod api {}
pub mod queries;
pub mod transactions;

pub type AccountId = AccountId32;
pub type Api = OnlineClient<ClientConfig>;
pub type BlockSubscription = Subscription<Header<u32, BlakeTwo256>>;
pub type Signer = PairSigner<ClientConfig, Keypair>;
pub type TransactionProgress = TxProgress<ClientConfig, Api>;
pub type TransactionStatus = SubTxStatus<ClientConfig, Api>;

pub type FilteredEvents<'a, T> = FilterEvents<
    'a,
    EventSubscription<ClientConfig, Api, Subscription<Header<u32, BlakeTwo256>>>,
    ClientConfig,
    T,
>;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TxStatus {
    Ready,
    Broadcast,
    InBlock,
    Finalized,
}

impl TxStatus {
    pub fn reached(self, status: &TransactionStatus) -> (bool, Option<TxHash>) {
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

pub fn pad_to_32_bytes(name: &[u8]) -> Result<[u8; 32], anyhow::Error> {
    let mut output = [0u8; 32];
    anyhow::ensure!(name.len() <= output.len(), "name too long");
    output.copy_from_slice(name);
    Ok(output)
}

pub struct Guild {
    pub name: [u8; 32],
    pub metadata: Vec<u8>,
    pub roles: Vec<Role>,
}

pub struct Role {
    pub name: [u8; 32],
    pub requirements: Vec<Requirement>,
}

#[cfg(test)]
mod test {
    use super::{TransactionStatus, TxHash, TxStatus};

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

        let status = TransactionStatus::Usurped(TxHash::default());
        let (reached, _) = flag.reached(&status);
        assert!(reached);
        let status = TransactionStatus::Retracted(TxHash::default());
        let (reached, _) = flag.reached(&status);
        assert!(reached);
        let status = TransactionStatus::FinalityTimeout(TxHash::default());
        let (reached, _) = flag.reached(&status);
        assert!(reached);
    }
}
