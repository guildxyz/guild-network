use ethers::core::k256::ecdsa::SigningKey;
use ethers::signers::{LocalWallet, Signer as EthSignerT};
use gn_api::tx::{self, Signer};
use gn_api::{Api, ClientConfig};
use sp_core::hashing::blake2_256;

use subxt::utils::{MultiAddress, MultiSignature};

use std::sync::Arc;

pub struct EthSigner {
    account_id: <ClientConfig as subxt::Config>::AccountId,
    wallet: LocalWallet,
}

impl EthSigner {
    pub fn from_seed(seed: [u8; 32]) -> Self {
        let wallet = LocalWallet::from(SigningKey::from_bytes(&seed).unwrap());
        let account_id =
            <ClientConfig as subxt::Config>::AccountId::from(blake2_256(wallet.address().as_ref()));
        Self { account_id, wallet }
    }
}

impl subxt::tx::Signer<ClientConfig> for EthSigner {
    fn account_id(&self) -> &<ClientConfig as subxt::Config>::AccountId {
        &self.account_id
    }

    fn address(&self) -> <ClientConfig as subxt::Config>::Address {
        MultiAddress::Address20(self.wallet.address().into())
    }

    fn sign(&self, signer_payload: &[u8]) -> <ClientConfig as subxt::Config>::Signature {
        futures::executor::block_on(async move {
            let signature = self.wallet.sign_message(signer_payload).await.unwrap();
            MultiSignature::Ecdsa(signature.into())
        })
    }
}

pub async fn eth(api: Api, _signer: Arc<Signer>) {
    todo!()
}
