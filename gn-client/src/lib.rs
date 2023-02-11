#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]

// re-exports
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
pub type Balance = u128;
pub type GuildCall = runtime::runtime_types::pallet_guild::pallet::Call;
pub type Index = u32;
pub type OracleRequest = runtime::oracle::events::OracleRequest;
pub type Request = gn_common::Request<AccountId>;
pub type RequestData = runtime::runtime_types::gn_common::RequestData<AccountId>;
pub type RuntimeIdentity = runtime::runtime_types::gn_common::identity::Identity;
pub type RuntimeIdentityWithAuth =
    runtime::runtime_types::gn_common::identity::auth::IdentityWithAuth;
pub type SubxtError = subxt::Error;
pub type TransactionProgress = subxt::tx::TxProgress<ClientConfig, Api>;
pub type TransactionStatus = subxt::tx::TxStatus<ClientConfig, Api>;

const PAD_BYTES: usize = 32;

use gn_common::identity::Identity;

pub fn id_rt2canon(input: RuntimeIdentity) -> Identity {
    unsafe { std::mem::transmute::<RuntimeIdentity, Identity>(input) }
}

pub fn id_canon2rt(input: Identity) -> RuntimeIdentity {
    unsafe { std::mem::transmute::<Identity, RuntimeIdentity>(input) }
}
