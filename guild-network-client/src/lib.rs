use subxt::{
    events::{EventSubscription, FilterEvents},
    ext::sp_runtime::{generic::Header, traits::BlakeTwo256, AccountId32},
    rpc::Subscription,
    tx::{PairSigner, TxProgress, TxStatus as SubTxStatus},
    OnlineClient,
};

// re-exports
pub use serde_cbor::{from_slice as cbor_deserialize, to_vec as cbor_serialize};
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
pub type GuildData = runtime::runtime_types::pallet_guild::pallet::GuildData<AccountId>;
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
