mod create_guild;
mod manage_role;
mod register;

pub use create_guild::create_guild;
pub use manage_role::manage_role;
pub use register::register;

use gn_client::transactions::{track_progress, TxStatus};
use gn_client::{
    Api, Hash, PreparedMsgWithParams, Signature, SrSignature, SubstrateAddress, SubxtError,
};

pub async fn send_tx(
    api: Api,
    substrate_address: SubstrateAddress,
    signature: &[u8],
    prepared: &PreparedMsgWithParams,
    tx_status: TxStatus,
) -> Result<Option<Hash>, SubxtError> {
    let sr_sig = SrSignature::from_slice(signature)
        .ok_or_else(|| SubxtError::Other("invalid signature bytes".to_string()))?;
    let signature = Signature::Sr25519(sr_sig);

    let mut progress = api
        .tx()
        .pack_and_submit_then_watch(substrate_address, signature, &prepared.encoded_params)
        .await?;

    track_progress(&mut progress, tx_status).await
}

#[cfg(test)]
mod test {
    use super::*;
    use gn_client::data::{Guild, Role};
    use gn_client::queries::{self, GuildFilter};
    use gn_client::{AccountKeyring, Api, Signature, Signer, TxSignerTrait};
    use gn_common::identities::Identity;
    use gn_common::pad::pad_to_32_bytes;
    use gn_common::requirements::{Requirement, RequirementsWithLogic};
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn all_transactions() {
        let guild_name = "yellow-guild";
        let role_name = "canary-role";
        let evm_address = [
            45, 48, 227, 6, 107, 2, 236, 154, 191, 183, 175, 121, 86, 27, 2, 236, 112, 110, 213, 74,
        ];
        let evm_signature = [
            96, 116, 11, 233, 146, 64, 97, 53, 128, 37, 159, 151, 138, 73, 148, 90, 83, 91, 237,
            104, 28, 186, 75, 75, 139, 202, 88, 250, 233, 60, 63, 126, 95, 180, 10, 64, 252, 128,
            147, 63, 252, 142, 192, 125, 104, 52, 76, 113, 228, 109, 237, 55, 74, 78, 35, 147, 185,
            84, 65, 144, 90, 70, 80, 9, 28,
        ];
        let signer = Signer::new(AccountKeyring::Alice.pair());
        let api = Api::from_url(gn_test_data::URL)
            .await
            .expect("failed to establish websocket connection");

        // 1) register alice with dummy identities
        let prepared = register(
            api.clone(),
            signer.account_id(),
            Some(hex::encode(evm_address)),
            Some(hex::encode(evm_signature)),
            Some(123.to_string()),
            None,
        )
        .await
        .expect("failed to get payload");

        let signature = match signer.sign(&prepared.prepared_msg) {
            Signature::Sr25519(sig) => sig.0,
            _ => panic!("should be sr signature"),
        };

        // send transaction
        let maybe_hash = send_tx(
            api.clone(),
            signer.address(),
            &signature,
            &prepared,
            TxStatus::InBlock,
        )
        .await
        .expect("failed to send tx");
        assert!(maybe_hash.is_some());

        // 2) create a guild with a single FREE role
        let guild = Guild {
            name: pad_to_32_bytes(guild_name),
            metadata: vec![1, 2, 3, 4, 5, 6, 7],
            roles: vec![Role {
                name: pad_to_32_bytes(role_name),
                reqs: RequirementsWithLogic {
                    logic: "0".to_string(),
                    requirements: vec![Requirement::Free],
                },
            }],
        };
        let prepared = create_guild(api.clone(), signer.account_id(), guild)
            .await
            .expect("failed to get payload");

        let signature = match signer.sign(&prepared.prepared_msg) {
            Signature::Sr25519(sig) => sig.0,
            _ => panic!("should be sr signature"),
        };

        let mut i = 0;
        loop {
            // send transaction
            let result = send_tx(
                api.clone(),
                signer.address(),
                &signature,
                &prepared,
                TxStatus::InBlock,
            )
            .await;
            if result.is_ok() {
                assert!(maybe_hash.is_some());
                break;
            } else if i == 100 {
                panic!("failed to send msg")
            }
            i += 1;
        }

        // 3) join the guild with the free role
        let prepared = manage_role(
            api.clone(),
            signer.account_id(),
            guild_name.to_string(),
            role_name.to_string(),
        )
        .await
        .expect("failed to generate payload");

        let signature = match signer.sign(&prepared.prepared_msg) {
            Signature::Sr25519(sig) => sig.0,
            _ => panic!("should be sr signature"),
        };

        let i = (0..1000).fold(0u32, |acc, x| acc + x * 2);
        println!("{i}");

        // send transaction
        let maybe_hash = send_tx(
            api.clone(),
            signer.address(),
            &signature,
            &prepared,
            TxStatus::InBlock,
        )
        .await
        .expect("failed to send tx");
        assert!(maybe_hash.is_some());

        // query members again in a loop (for some reason, send tx doesn't wait until it's included)
        loop {
            let members = queries::members(
                api.clone(),
                Some(&GuildFilter {
                    name: pad_to_32_bytes(guild_name),
                    role: Some(pad_to_32_bytes(role_name)),
                }),
                10,
            )
            .await
            .expect("failed to query members");

            if members.len() == 1 {
                assert_eq!(&members[0], signer.account_id());
                break;
            }
        }

        // last but not least register another (not-yet registered) telegram
        // identity
        let prepared = register(
            api.clone(),
            signer.account_id(),
            None,
            None,
            None,
            Some("456".to_string()),
        )
        .await
        .expect("failed to get payload");

        let signature = match signer.sign(&prepared.prepared_msg) {
            Signature::Sr25519(sig) => sig.0,
            _ => panic!("should be sr signature"),
        };

        // send transaction
        let maybe_hash = send_tx(
            api.clone(),
            signer.address(),
            &signature,
            &prepared,
            TxStatus::InBlock,
        )
        .await
        .expect("failed to send tx");
        assert!(maybe_hash.is_some());

        // query members again in a loop (for some reason, send tx doesn't wait until it's included)
        loop {
            let identities = queries::user_identity(api.clone(), signer.account_id())
                .await
                .expect("failed to query identities");

            if identities.len() == 3 {
                assert!(identities.contains(&Identity::Discord(123)));
                assert!(identities.contains(&Identity::Telegram(456)));
                break;
            }
        }
    }
}
