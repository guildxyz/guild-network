use ethers::core::k256::ecdsa::SigningKey;
use ethers::signers::{LocalWallet, Signer as EthSignerT};
use gn_api::tx::{self, Signer};
use gn_api::{query, Api, ClientConfig};
use parity_scale_codec::Encode;
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
    let primary = Arc::new(EthSigner::from_seed([2u8; 32]));
    let linked = Arc::new(EthSigner::from_seed([3u8; 32]));
    let wallet = gn_sig::webcrypto::wallet::Wallet::from_seed([10u8; 32]).unwrap();
    let authority = gn_sig::webcrypto::hash_pubkey(&wallet.pubkey());
    let prefix = [98u8; 8];

    // create a guild just for fun
    let guild_name = [111u8; 32];
    let payload = tx::guild::create_guild(guild_name, vec![1, 2, 3]);
    tx::send::in_block(api.clone(), &payload, Arc::clone(&primary))
        .await
        .unwrap();

    // check that guild is indeed created
    let guild_id = query::guild::guild_id(api.clone(), guild_name)
        .await
        .unwrap();
    println!("CREATED GUILD: {:?}", guild_id);

    // register primary address
    let payload = tx::identity::register();
    tx::send::in_block(api.clone(), &payload, Arc::clone(&primary))
        .await
        .expect("failed to send tx");

    // authorize wallet
    let payload = tx::identity::authorize(authority, false);
    tx::send::in_block(api.clone(), &payload, Arc::clone(&primary))
        .await
        .expect("failed to send tx");

    let authorities = query::identity::authorities(api.clone(), primary.as_ref().account_id())
        .await
        .expect("failed to dispatch query");

    assert_eq!(authorities, [authority, [0u8; 32]]);

    // link address
    let signature = wallet.sign(linked.account_id().encode()).unwrap();
    let payload = tx::identity::link_address(primary.account_id().clone(), prefix, signature);
    tx::send::in_block(api.clone(), &payload, Arc::clone(&linked))
        .await
        .expect("failed to send tx");

    // check addresses
    let address_map = query::identity::addresses(api, primary.as_ref().account_id())
        .await
        .expect("failed to dispatch query");

    let expected = vec![linked.as_ref().account_id().clone()];
    assert_eq!(address_map.get(&prefix), Some(&expected));

    println!("LINKED ADDRESS: {}", expected[0]);
}
