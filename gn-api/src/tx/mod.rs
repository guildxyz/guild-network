mod payload;
pub mod send;
mod status;

pub use payload::*;
pub use sp_core::crypto::Pair as PairT;
pub use sp_core::sr25519::Pair as Keypair;
pub use status::TxStatus;
pub use subxt::tx::Signer as SignerT;

pub type Signer = subxt::tx::PairSigner<ClientConfig, Keypair>;

use crate::{Api, ClientConfig, SubxtError};
use std::sync::Arc;

pub async fn api_with_signer(
    url: String,
    seed: &str,
    password: Option<&str>,
) -> Result<(Api, Arc<Signer>), SubxtError> {
    let api = Api::from_url(url).await?;
    let signer = signer(seed, password)?;
    Ok((api, signer))
}

pub fn signer(seed: &str, password: Option<&str>) -> Result<Arc<Signer>, SubxtError> {
    let keypair =
        Keypair::from_string(seed, password).map_err(|e| SubxtError::Other(e.to_string()))?;
    Ok(Arc::new(Signer::new(keypair)))
}
