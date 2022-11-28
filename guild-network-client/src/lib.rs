use guild_network_common::{GuildName, RoleName};
use subxt::{
    events::{EventSubscription, FilterEvents},
    ext::sp_runtime::{generic::Header, traits::BlakeTwo256, AccountId32},
    rpc::Subscription,
    tx::{PairSigner, TxProgress, TxStatus as SubTxStatus},
    OnlineClient,
};

// re-exports
pub use sp_keyring::sr25519::sr25519::Pair as Keypair;
pub use subxt::ext::sp_core::H256 as Hash;
pub use subxt::tx::Signer as TxSignerTrait;
pub use subxt::PolkadotConfig as ClientConfig;

#[subxt::subxt(runtime_metadata_path = "./artifacts/metadata.scale")]
pub mod runtime {}
pub mod data;
pub mod queries;
pub mod transactions;

pub type AccountId = AccountId32;
pub type Api = OnlineClient<ClientConfig>;
pub type BalanceOf = u128;
pub type BlockHeader = Header<BlockNumber, BlakeTwo256>;
pub type BlockNumber = u32;
pub type BlockSubscription = Subscription<BlockHeader>;
pub type JoinRequest = guild_network_common::JoinRequest<AccountId>;
pub type GuildCall = runtime::runtime_types::pallet_guild::pallet::Call;
pub type GuildData = runtime::runtime_types::pallet_guild::pallet::Guild<AccountId>;
pub type OracleRequest = runtime::runtime_types::pallet_chainlink::pallet::GenericRequest<
    AccountId,
    GuildCall,
    u32,
    u128,
>;
pub type Signer = PairSigner<ClientConfig, Keypair>;
pub type TransactionProgress = TxProgress<ClientConfig, Api>;
pub type TransactionStatus = SubTxStatus<ClientConfig, Api>;

pub type FilteredEvents<'a, T> =
    FilterEvents<'a, EventSubscription<ClientConfig, Api, BlockSubscription>, ClientConfig, T>;

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

pub fn signed_msg(id: &AccountId, guild_name: &GuildName, role_name: &RoleName) -> String {
    format!(
        "{} wanna join role {:?} of guild {:?}",
        id, guild_name, role_name
    )
}

pub fn pad_to_32_bytes(name: &[u8]) -> Result<[u8; 32], anyhow::Error> {
    let mut output = [0u8; 32];
    anyhow::ensure!(name.len() <= output.len(), "name too long");
    output[..name.len()].copy_from_slice(name);
    Ok(output)
}

#[cfg(test)]
mod test {
    use super::pad_to_32_bytes;
    use super::{Hash, TransactionStatus, TxStatus};

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

    #[test]
    fn pad_bytes() {
        let bytes = b"hello";
        let padded = pad_to_32_bytes(bytes).unwrap();
        assert_eq!(&padded[..5], bytes);
        assert_eq!(&padded[5..], &[0u8; 27]);

        let bytes = &[123; 32];
        let padded = pad_to_32_bytes(bytes).unwrap();
        assert_eq!(&padded, bytes);

        let bytes = &[12; 33];
        assert!(pad_to_32_bytes(bytes).is_err());
    }
}
