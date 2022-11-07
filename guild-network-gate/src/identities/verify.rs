use super::{Identity, IdentityAuth};
use anyhow::anyhow;
use ethers_core::types::Signature as EthSignature;

impl Identity {
    pub fn verify(&self, auth: &IdentityAuth) -> Result<(), anyhow::Error> {
        let is_valid = match (self, auth) {
            (Self::EvmChain(address), IdentityAuth::EvmChain { signature, msg }) => {
                let msg = std::str::from_utf8(msg).map_err(|e| anyhow!(e))?;
                let ethers_signature =
                    EthSignature::try_from(signature.as_bytes()).map_err(|e| anyhow!(e))?;
                ethers_signature
                    .verify(msg, address.to_fixed_bytes())
                    .map_err(|e| anyhow!(e))?;
                true
            }
            (Self::Discord(_), _) => true,
            (Self::Telegram(_), _) => true,
            // could return an error if we want but this arm means a platform
            // mismatch between the identity and the verification data
            _ => false,
        };

        anyhow::ensure!(is_valid, "invalid identity");
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::address;
    use ethereum_types::{Address, Signature};
    use std::str::FromStr;

    fn valid_signature(msg: &str, signature: Signature, address: Address) -> bool {
        let identity = Identity::EvmChain(address);
        let identity_auth = IdentityAuth::EvmChain {
            signature,
            msg: msg.as_bytes().to_owned(),
        };
        identity.verify(&identity_auth).is_ok()
    }

    #[test]
    fn valid_evm_signature() {
        let msg = "requiem aeternam dona eis";
        let signature = Signature::from_str(
"fa2759679db3b02dae5e3627572a282d20e98f2eb142b86edc70b10352a1a26e6249f5350f02e1210e9aa57c9a78e6ae3eb380f7b32ea144f12614baffba16711b").unwrap();
        let address = address!("0x9d5eba47309d5ddbc0823a878c5960c2aad86fa6");
        assert!(valid_signature(msg, signature, address));
    }

    #[test]
    fn invalid_evm_signature_wrong_msg() {
        let msg = "invalid";
        let signature = Signature::from_str(
"fa2759679db3b02dae5e3627572a282d20e98f2eb142b86edc70b10352a1a26e6249f5350f02e1210e9aa57c9a78e6ae3eb380f7b32ea144f12614baffba16711b").unwrap();
        let address = address!("0x9d5eba47309d5ddbc0823a878c5960c2aad86fa6");
        assert!(!valid_signature(msg, signature, address));
    }

    #[test]
    fn invalid_evm_signature_wrong_address() {
        let msg = "requiem aeternam dona eis";
        let signature = Signature::from_str(
"fa2759679db3b02dae5e3627572a282d20e98f2eb142b86edc70b10352a1a26e6249f5350f02e1210e9aa57c9a78e6ae3eb380f7b32ea144f12614baffba16711b").unwrap();
        let address = address!("0x3d5eba47309d5ddbc0823a878c5960c2aad86fa6");
        assert!(!valid_signature(msg, signature, address));
    }

    #[test]
    fn invalid_evm_signature_wrong_signature() {
        let msg = "requiem aeternam dona eis";
        let signature = Signature::from_str(
"aa2759679db3b02dae5e3627572a282d20e98f2eb142b86edc70b10352a1a26e6249f5350f02e1210e9aa57c9a78e6ae3eb380f7b32ea144f12614baffba16711b").unwrap();
        let address = address!("0x9d5eba47309d5ddbc0823a878c5960c2aad86fa6");
        assert!(!valid_signature(msg, signature, address));
    }

    #[test]
    fn verify_off_chain_platforms() {
        let discord_id = Identity::Discord(vec![]);
        let telegram_id = Identity::Telegram(vec![]);

        let discord_auth = IdentityAuth::Discord;
        let telegram_auth = IdentityAuth::Telegram;

        assert!(discord_id.verify(&discord_auth).is_ok());
        assert!(telegram_id.verify(&telegram_auth).is_ok());
    }
}
