use ethers::signers::Signer as EthSignerT;
use gn_api::{Api, ClientConfig};
use gn_api::tx::{self, Signer};

use std::sync::Arc;

struct EthSigner(ethers::signers::LocalWallet);

impl subxt::tx::Signer<ClientConfig> for EthSigner {
    fn account_id(&self) -> &<ClientConfig as subxt::Config>::AccountId {
        todo!()
    }

    fn address(&self) -> <ClientConfig as subxt::Config>::Address {
        todo!()
    }

    fn sign(&self, signer_payload: &[u8]) -> <ClientConfig as subxt::Config>::Signature {
        todo!()
    }
}

pub async fn eth(api: Api, _signer: Arc<Signer>) {
    todo!()
}
