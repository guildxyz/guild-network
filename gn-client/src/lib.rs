#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]

// re-exports
pub use subxt::utils::MultiAddress;
pub use subxt::utils::H256 as Hash;
pub use subxt::PolkadotConfig as ClientConfig;

#[subxt::subxt(runtime_metadata_path = "./artifacts/metadata.scale")]
pub mod runtime {}
pub mod query;
//#[cfg(feature = "tx")]
//pub mod tx;

pub type Api = subxt::OnlineClient<ClientConfig>;
pub type AccountId = subxt::utils::AccountId32;
pub type Balance = u128;
pub type GuildCall = runtime::runtime_types::pallet_guild::pallet::Call;
pub type Index = u32;
pub type OracleRequest = runtime::oracle::events::OracleRequest;
pub type Request = gn_common::Request<AccountId>;
pub type RuntimeGuild = runtime::runtime_types::gn_common::Guild<AccountId>;
pub type RuntimeIdentity = runtime::runtime_types::gn_common::identity::Identity;
pub type RuntimeIdentityWithAuth =
    runtime::runtime_types::gn_common::identity::auth::IdentityWithAuth;
pub type RuntimeRole = runtime::runtime_types::gn_common::Role<Hash>;
pub type SubxtError = subxt::Error;
pub type TransactionProgress = subxt::tx::TxProgress<ClientConfig, Api>;
pub type TransactionStatus = subxt::tx::TxStatus<ClientConfig, Api>;

const PAD_BYTES: usize = 32;

use gn_common::identity::Identity;
use gn_common::{Guild, Role};

pub fn from_runtime_id(input: RuntimeIdentity) -> Identity {
    unsafe { std::mem::transmute::<RuntimeIdentity, Identity>(input) }
}

pub fn from_runtime_guild(input: RuntimeGuild) -> Guild<AccountId> {
    unsafe { std::mem::transmute::<RuntimeGuild, Guild<AccountId>>(input) }
}

pub fn from_runtime_role(input: RuntimeRole) -> Role<Hash> {
    unsafe { std::mem::transmute::<RuntimeRole, Role<Hash>>(input) }
}
