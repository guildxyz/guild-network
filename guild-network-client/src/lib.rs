use guild_network_gate::requirements::Requirement;
use subxt::{
    config::{SubstrateConfig, WithExtrinsicParams},
    events::{EventSubscription, FilterEvents},
    ext::sp_runtime::{generic::Header, traits::BlakeTwo256, AccountId32},
    rpc::Subscription,
    tx::{BaseExtrinsicParams, PairSigner, PlainTip, TxProgress, TxStatus as SubTxStatus, TxInBlock},
    OnlineClient,
};

// re-exports
pub use sp_keyring::sr25519::sr25519::Pair as Keypair;
pub use subxt::tx::Signer as TxSignerTrait;
pub use subxt::PolkadotConfig as ClientConfig;

#[subxt::subxt(runtime_metadata_path = "./artifacts/metadata.scale")]
pub mod api {}
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
    pub fn reached(self, status: &TransactionStatus) -> bool {
        let mut reached = false;
        match status {
            TransactionStatus::Future => {},
            TransactionStatus::Ready => if self == Self::Ready { reached = true },
            TransactionStatus::Broadcast(_) => if self <= Self::Broadcast { reached = true },
            TransactionStatus::InBlock(_) => if self <= Self::InBlock { reached = true },
            TransactionStatus::Finalized(_) => if self <= Self::Finalized { reached = true },
            _ => reached = true, // these arms represent failed transactions which won't advance
        }
        reached
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
    use super::{TxStatus, TransactionStatus};
    use subxt::ext::sp_core::H256;

    #[test]
    fn tx_status_reached() {
        let flag = TxStatus::Ready;

        let status = TransactionStatus::Future;
        assert!(!flag.reached(&status));
        let status = TransactionStatus::Ready;
        assert!(flag.reached(&status));

        let flag = TxStatus::Broadcast;

        assert!(!flag.reached(&status));
        let status = TransactionStatus::Broadcast(vec![]);
        assert!(flag.reached(&status));

        let flag = TxStatus::InBlock;

        let status = TransactionStatus::Usurped(H256::default());
        assert!(flag.reached(&status));
        let status = TransactionStatus::Retracted(H256::default());
        assert!(flag.reached(&status));
        let status = TransactionStatus::FinalityTimeout(H256::default());
        assert!(flag.reached(&status));
    }
}
