#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]

// re-exports
pub use subxt::utils::MultiAddress;
pub use subxt::utils::H256;
pub use subxt::{Config as ConfigT, SubstrateConfig as ClientConfig};

mod cast;
#[subxt::subxt(runtime_metadata_path = "./artifacts/metadata.scale")]
pub mod runtime {}
pub mod query;
#[cfg(feature = "tx")]
pub mod tx;

pub type Api = subxt::OnlineClient<ClientConfig>;
pub type AccountId = <ClientConfig as ConfigT>::AccountId;
pub type Balance = u128;
pub type GuildCall = runtime::runtime_types::pallet_guild::pallet::Call;
pub type OracleRequest = runtime::oracle::events::OracleRequest;
pub type Request = gn_common::Request<AccountId>;
pub type SessionKeys = runtime::runtime_types::gn_runtime::opaque::SessionKeys;
pub type SubxtError = subxt::Error;
pub type TransactionProgress = subxt::tx::TxProgress<ClientConfig, Api>;
pub type TransactionStatus = subxt::tx::TxStatus<ClientConfig, Api>;
