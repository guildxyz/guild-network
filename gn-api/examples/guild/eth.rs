use ethers::core::k256::ecdsa::SigningKey;
use ethers::signers::{LocalWallet, Signer as EthSignerT};
use gn_api::tx::{self, Signer};
use gn_api::{query, Api, ClientConfig};

use subxt::utils::{MultiAddress, MultiSignature};

use std::sync::Arc;

pub struct EthSigner {
    account_id: <ClientConfig as subxt::Config>::AccountId,
    wallet: LocalWallet,
}

impl EthSigner {
    pub fn from_seed(seed: [u8; 32]) -> Self {
        let wallet = LocalWallet::from(SigningKey::from_bytes(&seed).unwrap());
        let account_id = <ClientConfig as subxt::Config>::AccountId::from(gn_sig::address2account(
            wallet.address().to_fixed_bytes(),
        ));
        Self { account_id, wallet }
    }

    pub fn account_id(&self) -> &<ClientConfig as subxt::Config>::AccountId {
        &self.account_id
    }
}

impl subxt::tx::Signer<ClientConfig> for EthSigner {
    fn account_id(&self) -> &<ClientConfig as subxt::Config>::AccountId {
        &self.account_id
    }

    fn address(&self) -> <ClientConfig as subxt::Config>::Address {
        MultiAddress::Id(self.account_id.clone())
    }

    fn sign(&self, signer_payload: &[u8]) -> <ClientConfig as subxt::Config>::Signature {
        futures::executor::block_on(async move {
            let mut signature: [u8; 65] = self
                .wallet
                .sign_message(signer_payload)
                .await
                .unwrap()
                .into();
            if signature[64] >= 27 {
                signature[64] -= 27;
            }
            MultiSignature::Ecdsa(signature)
        })
    }
}

pub async fn eth(api: Api, _signer: Arc<Signer>) {
    let eth_signer = Arc::new(EthSigner::from_seed([2u8; 32]));
    let guild_name = [111u8; 32];
    let payload = tx::guild::create_guild(guild_name, vec![1, 2, 3]);
    tx::send::in_block(api.clone(), &payload, Arc::clone(&eth_signer))
        .await
        .unwrap();

    let guild_id = query::guild::guild_id(api, guild_name).await.unwrap();
    println!("{:?}", guild_id);
}
