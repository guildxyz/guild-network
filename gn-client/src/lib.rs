//    events::{EventSubscription, FilterEvents},
//    ext::sp_runtime::{generic::Header, traits::BlakeTwo256, AccountId32},
//    rpc::Subscription,
//    tx::{PairSigner, TxProgress, TxStatus as SubTxStatus},
//    OnlineClient,
//};
//
//// re-exports
pub use serde_cbor::{from_slice as cbor_deserialize, to_vec as cbor_serialize};
pub use subxt::utils::MultiAddress;
pub use subxt::utils::H256 as Hash;
pub use subxt::PolkadotConfig as ClientConfig;

#[subxt::subxt(runtime_metadata_path = "./artifacts/metadata.scale")]
pub mod runtime {}
pub mod data;
pub mod query;
#[cfg(feature = "tx")]
pub mod tx;

pub type Api = subxt::OnlineClient<ClientConfig>;
pub type AccountId = subxt::utils::AccountId32;
//pub type SubstrateAddress = <ClientConfig as subxt::Config>::Address;
//pub type BalanceOf = u128;
//pub type BlockHeader = Header<BlockNumber, BlakeTwo256>;
//pub type BlockNumber = u32;
//pub type BlockSubscription = Subscription<BlockHeader>;
//pub type GuildCall = runtime::runtime_types::pallet_guild::pallet::Call;
//pub type Index = u32;
//pub type OracleRequest = runtime::runtime_types::pallet_oracle::pallet::GenericRequest<
//    AccountId,
//    GuildCall,
//    u32,
//    BalanceOf,
//>;
pub type Request = gn_common::Request<AccountId>;
pub type RequestData = runtime::runtime_types::gn_common::RequestData<AccountId>;
pub type RuntimeIdentity = runtime::runtime_types::gn_common::identities::Identity;
pub type RuntimeIdentityWithAuth = runtime::runtime_types::gn_common::identities::IdentityWithAuth;
//pub type Signature = <ClientConfig as subxt::Config>::Signature;
pub type SubxtError = subxt::Error;
pub type TransactionProgress = subxt::tx::TxProgress<ClientConfig, Api>;
pub type TransactionStatus = subxt::tx::TxStatus<ClientConfig, Api>;
//
//pub type FilteredEvents<'a, T> =
//    FilterEvents<'a, EventSubscription<ClientConfig, Api, BlockSubscription>, ClientConfig, T>;
