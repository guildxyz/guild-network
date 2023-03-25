use super::{Identity as CliIdentity, QUERY_ERROR, TX_ERROR};
use gn_api::{
    query,
    tx::{self, Signer},
    Api,
};
use gn_common::identity::{Identity, IdentityWithAuth};
use gn_common::merkle::Proof as MerkleProof;
use gn_common::pad::pad_to_n_bytes;

use std::sync::Arc;

pub struct ProofIndices {
    pub id: u8,
    pub leaf: usize,
}

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

    tx::send::ready(api, &payload, signer)
        .await
        .expect(TX_ERROR);
}

pub async fn join(
    api: Api,
    signer: Arc<Signer>,
    guild: String,
    role: String,
    maybe_indices: Option<ProofIndices>,
) {
    let guild = pad_to_n_bytes::<32, _>(&guild);
    let role = pad_to_n_bytes::<32, _>(&role);
    let proof = if let Some(indices) = maybe_indices {
        query::allowlist(api.clone(), guild, role)
            .await
            .expect(QUERY_ERROR)
            .map(|allowlist| MerkleProof::new(&allowlist, indices.leaf, indices.id))
    } else {
        log::info!("no proof generated");
        None
    };

    let payload = tx::join(guild, role, proof);

    tx::send::ready(api, &payload, signer)
        .await
        .expect(TX_ERROR);
}
