use ethers::types::{Address, Signature as EvmSignature};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Identity {
    EvmChain(Address),
    Discord(Vec<u8>),
    Telegram(Vec<u8>),
}

#[derive(Serialize, Deserialize)]
pub enum IdentityAuth {
    EvmChain {
        signature: EvmSignature,
        msg: Vec<u8>,
    },
    Discord,  // not authenticating for now
    Telegram, // not authenticating for now
}

impl Identity {
    pub fn verify(&self, auth: &IdentityAuth) -> bool {
        match (self, auth) {
            (Self::EvmChain(address), IdentityAuth::EvmChain { signature, msg }) => {
                if let Ok(msg) = std::str::from_utf8(msg) {
                    signature.verify(msg, *address).is_ok()
                } else {
                    false
                }
            }
            (Self::Discord(_), _) => true,
            (Self::Telegram(_), _) => true,
            // could return an error if we want but this arm means a platform
            // mismatch between the identity and the verification data
            _ => false,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ethers::prelude::k256::elliptic_curve::rand_core;
    use ethers::prelude::LocalWallet;

    use ethers::signers::Signer;

    #[tokio::test]
    async fn valid_evm_signature() {
        let wallet = LocalWallet::new(&mut rand_core::OsRng);
        let msg = "requiem aeternam dona eis";
        let signature = wallet.sign_message(msg).await.unwrap();
        let identity = Identity::EvmChain(wallet.address());
        let identity_auth = IdentityAuth::EvmChain {
            signature,
            msg: msg.as_bytes().to_owned(),
        };
        assert!(identity.verify(&identity_auth));
    }

    #[tokio::test]
    async fn invalid_evm_signature() {
        let wallet = LocalWallet::new(&mut rand_core::OsRng);
        let msg = "requiem aeternam dona eis";
        let signature = wallet.sign_message(msg).await.unwrap();
        let identity = Identity::EvmChain(wallet.address());

        let identity_auth = IdentityAuth::EvmChain {
            signature,
            msg: "invalid".as_bytes().to_owned(),
        };
        assert!(!identity.verify(&identity_auth));
    }

    #[tokio::test]
    async fn verify_off_chain_platforms() {
        let discord_id = Identity::Discord(vec![]);
        let telegram_id = Identity::Telegram(vec![]);

        let discord_auth = IdentityAuth::Discord;
        let telegram_auth = IdentityAuth::Telegram;

        assert!(discord_id.verify(&discord_auth));
        assert!(telegram_id.verify(&telegram_auth));
    }
}
