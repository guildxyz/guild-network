use super::*;
use anyhow::anyhow;
use ethers_core::types::Signature as EthSignature;

impl IdentityWithAuth {
    pub fn verify(&self, verification_msg: &str) -> Result<(), anyhow::Error> {
        match self {
            Self::EvmChain(address, signature) => {
                // NOTE unwrap is fine because of fixed length array
                let ethers_signature = EthSignature::try_from(signature.as_ref()).unwrap();
                ethers_signature
                    .verify(verification_msg.as_bytes(), address)
                    .map_err(|e| anyhow!(e))
            }
            Self::Discord(_, _) => Ok(()),
            Self::Telegram(_, _) => Ok(()),
        }
    }

    pub fn into_platform_with_id(self) -> (Platform, Identity) {
        match self {
            Self::EvmChain(address, _) => (Platform::EvmChain, Identity::EvmChain(address)),
            Self::Discord(id, _) => (Platform::Discord, Identity::Discord(id)),
            Self::Telegram(id, _) => (Platform::Telegram, Identity::Telegram(id)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::*;

    fn valid_signature(msg: &str, signature: EvmSignature, address: EvmAddress) -> bool {
        let identity = IdentityWithAuth::EvmChain(address, signature);
        identity.verify(msg).is_ok()
    }

    #[test]
    fn valid_evm_signature() {
        let msg = "requiem aeternam dona eis";
        let address = [
            157, 94, 186, 71, 48, 157, 93, 219, 192, 130, 58, 135, 140, 89, 96, 194, 170, 216, 111,
            166,
        ];
        let signature = [
            250, 39, 89, 103, 157, 179, 176, 45, 174, 94, 54, 39, 87, 42, 40, 45, 32, 233, 143, 46,
            177, 66, 184, 110, 220, 112, 177, 3, 82, 161, 162, 110, 98, 73, 245, 53, 15, 2, 225,
            33, 14, 154, 165, 124, 154, 120, 230, 174, 62, 179, 128, 247, 179, 46, 161, 68, 241,
            38, 20, 186, 255, 186, 22, 113, 27,
        ];
        assert!(valid_signature(msg, signature, address));
    }

    #[test]
    fn invalid_evm_signature_wrong_msg() {
        let msg = "invalid";
        let address = [
            157, 94, 186, 71, 48, 157, 93, 219, 192, 130, 58, 135, 140, 89, 96, 194, 170, 216, 111,
            166,
        ];
        let signature = [
            250, 39, 89, 103, 157, 179, 176, 45, 174, 94, 54, 39, 87, 42, 40, 45, 32, 233, 143, 46,
            177, 66, 184, 110, 220, 112, 177, 3, 82, 161, 162, 110, 98, 73, 245, 53, 15, 2, 225,
            33, 14, 154, 165, 124, 154, 120, 230, 174, 62, 179, 128, 247, 179, 46, 161, 68, 241,
            38, 20, 186, 255, 186, 22, 113, 27,
        ];
        assert!(!valid_signature(msg, signature, address));
    }

    #[test]
    fn invalid_evm_signature_wrong_address() {
        let msg = "requiem aeternam dona eis";
        let address = [0; 20];
        let signature = [
            250, 39, 89, 103, 157, 179, 176, 45, 174, 94, 54, 39, 87, 42, 40, 45, 32, 233, 143, 46,
            177, 66, 184, 110, 220, 112, 177, 3, 82, 161, 162, 110, 98, 73, 245, 53, 15, 2, 225,
            33, 14, 154, 165, 124, 154, 120, 230, 174, 62, 179, 128, 247, 179, 46, 161, 68, 241,
            38, 20, 186, 255, 186, 22, 113, 27,
        ];
        assert!(!valid_signature(msg, signature, address));
    }

    #[test]
    fn invalid_evm_signature_wrong_signature() {
        let msg = "requiem aeternam dona eis";
        let address = [
            157, 94, 186, 71, 48, 157, 93, 219, 192, 130, 58, 135, 140, 89, 96, 194, 170, 216, 111,
            166,
        ];
        let signature = [0; 65];
        assert!(!valid_signature(msg, signature, address));
    }

    #[test]
    fn verify_off_chain_platforms() {
        let discord_id = IdentityWithAuth::Discord(0, ());
        let telegram_id = IdentityWithAuth::Telegram(1, ());

        assert!(discord_id.verify("").is_ok());
        assert!(telegram_id.verify("").is_ok());
    }
}
