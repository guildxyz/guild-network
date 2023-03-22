use super::Identity as CliIdentity;
use gn_client::{
    tx::{self, Signer},
    Api,
};
use gn_common::identity::{Identity, IdentityWithAuth};
use gn_common::pad::pad_to_n_bytes;

use std::sync::Arc;

pub async fn register_identity(api: Api, signer: Arc<Signer>, identity: CliIdentity) {
    let payload = match identity {
        CliIdentity::Discord { id, index } => {
            let padded_id = pad_to_n_bytes::<64, _>(&format!("discord:{}", id));
            let identity_with_auth = IdentityWithAuth::Other(Identity::Other(padded_id), [0u8; 64]);
            tx::register(identity_with_auth, index)
        }
        CliIdentity::Telegram { id, index } => {
            let padded_id = pad_to_n_bytes::<64, _>(&format!("telegram:{}", id));
            let identity_with_auth = IdentityWithAuth::Other(Identity::Other(padded_id), [0u8; 64]);
            tx::register(identity_with_auth, index)
        }
        CliIdentity::Evm {
            address,
            signature,
            index,
        } => {
            let identity_with_auth =
                IdentityWithAuth::from_evm(&address, &signature).expect("invalid parameters");
            tx::register(identity_with_auth, index)
        }
    };

    tx::send::in_block(api, &payload, signer)
        .await
        .expect("failed to send tx");
}

#[allow(unused)]
pub async fn join(api: Api, signer: Arc<Signer>, guild: String, role: String) {}
