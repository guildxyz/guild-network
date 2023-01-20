mod signer;
mod transactions;

use signer::WasmSigner;

use gn_client::data::Guild;
use gn_client::queries::{self, GuildFilter};
use gn_client::transactions::TxStatus;
use gn_client::{AccountId, Api};
use gn_common::{pad::pad_to_32_bytes, utils, GuildName};
pub use serde_cbor::to_vec as cbor_serialize;
use serde_wasm_bindgen::{from_value as deserialize_from_value, to_value as serialize_to_value};
use wasm_bindgen::prelude::*;

use std::str::FromStr;

#[wasm_bindgen(js_name = "queryMembers")]
pub async fn query_members(guild: String, role: String, url: String) -> Result<JsValue, JsValue> {
    let api = Api::from_url(&url)
        .await
        .map_err(|e| JsValue::from(e.to_string()))?;

    let mut guild_filter: Option<GuildFilter> = None;

    if !guild.is_empty() && guild.len() < 32 {
        let guild_name = pad_to_32_bytes(&guild);
        let role_name = if !role.is_empty() && role.len() < 32 {
            Some(pad_to_32_bytes(&role))
        } else {
            None
        };

        guild_filter = Some(GuildFilter {
            name: guild_name,
            role: role_name,
        });
    }

    let members = queries::members(api, guild_filter.as_ref(), 10)
        .await
        .map_err(|e| JsValue::from(e.to_string()))?;

    serialize_to_value(&members).map_err(|e| JsValue::from(e.to_string()))
}

#[wasm_bindgen(js_name = "queryGuilds")]
pub async fn query_guilds(guild: String, url: String) -> Result<JsValue, JsValue> {
    let api = Api::from_url(&url)
        .await
        .map_err(|e| JsValue::from(e.to_string()))?;

    let mut guild_name: Option<GuildName> = None;
    if !guild.is_empty() && guild.len() < 32 {
        guild_name = Some(pad_to_32_bytes(&guild));
    }

    let guilds = queries::guilds(api, guild_name, 10)
        .await
        .map_err(|e| JsValue::from(e.to_string()))?;

    serialize_to_value(&guilds).map_err(|e| JsValue::from(e.to_string()))
}

#[wasm_bindgen(js_name = "queryRequirements")]
pub async fn query_requirements(
    guild: String,
    role: String,
    url: String,
) -> Result<JsValue, JsValue> {
    let api = Api::from_url(&url)
        .await
        .map_err(|e| JsValue::from(e.to_string()))?;

    if guild.len() > 32 || role.len() > 32 {
        Err(JsValue::from("too long input name"))
    } else {
        let guild_name = pad_to_32_bytes(&guild);
        let role_name = pad_to_32_bytes(&role);

        let requirements = queries::requirements(api, guild_name, role_name)
            .await
            .map_err(|e| JsValue::from(e.to_string()))?;

        serialize_to_value(&requirements).map_err(|e| JsValue::from(e.to_string()))
    }
}

#[wasm_bindgen(js_name = "queryUserIdentity")]
pub async fn query_user_identity(address: String, url: String) -> Result<JsValue, JsValue> {
    let id = AccountId::from_str(&address).map_err(|e| JsValue::from(e.to_string()))?;
    let api = Api::from_url(&url)
        .await
        .map_err(|e| JsValue::from(e.to_string()))?;

    let identities = queries::user_identity(api, &id)
        .await
        .map_err(|e| JsValue::from(e.to_string()))?;

    serialize_to_value(&identities).map_err(|e| JsValue::from(e.to_string()))
}

#[wasm_bindgen(js_name = "verificationMsg")]
pub async fn verification_msg(address: String) -> String {
    utils::verification_msg(address)
}

#[wasm_bindgen(js_name = "registerTx")]
pub async fn register_tx(
    evm_address: Option<String>,
    evm_signature: Option<String>,
    discord: Option<String>,
    telegram: Option<String>,
    url: String,
) -> Result<JsValue, JsValue> {
    let api = Api::from_url(&url)
        .await
        .map_err(|e| JsValue::from(e.to_string()))?;

    let signer = WasmSigner::new().await.map_err(JsValue::from)?;

    let prepared = transactions::register(
        api.clone(),
        signer.account_id(),
        evm_address,
        evm_signature,
        discord,
        telegram,
    )
    .await
    .map_err(|e| JsValue::from(e.to_string()))?;

    let signature = signer.sign(&prepared.prepared_msg).await?;

    let maybe_hash = transactions::send_tx(
        api,
        signer.address().clone(),
        &signature,
        &prepared,
        TxStatus::InBlock,
    )
    .await
    .map_err(|e| JsValue::from(e.to_string()))?;

    serialize_to_value(&maybe_hash).map_err(|e| JsValue::from(e.to_string()))
}

#[wasm_bindgen(js_name = "joinGuildTxPayload")]
pub async fn join_guild_tx_payload(
    guild: String,
    role: String,
    url: String,
) -> Result<JsValue, JsValue> {
    let api = Api::from_url(&url)
        .await
        .map_err(|e| JsValue::from(e.to_string()))?;

    let signer = WasmSigner::new().await.map_err(JsValue::from)?;

    let prepared = transactions::manage_role(api.clone(), signer.account_id(), guild, role)
        .await
        .map_err(|e| JsValue::from(e.to_string()))?;

    let signature = signer.sign(&prepared.prepared_msg).await?;

    let maybe_hash = transactions::send_tx(
        api,
        signer.address().clone(),
        &signature,
        &prepared,
        TxStatus::InBlock,
    )
    .await
    .map_err(|e| JsValue::from(e.to_string()))?;

    serialize_to_value(&maybe_hash).map_err(|e| JsValue::from(e.to_string()))
}

#[wasm_bindgen(js_name = "createGuildTxPayload")]
pub async fn create_guild_tx_payload(guild: JsValue, url: String) -> Result<JsValue, JsValue> {
    let api = Api::from_url(&url)
        .await
        .map_err(|e| JsValue::from(e.to_string()))?;
    let signer = WasmSigner::new().await.map_err(JsValue::from)?;

    let guild = deserialize_from_value(guild).map_err(|e| JsValue::from(e.to_string()))?;

    let prepared = transactions::create_guild(api.clone(), signer.account_id(), guild)
        .await
        .map_err(|e| JsValue::from(e.to_string()))?;

    let signature = signer.sign(&prepared.prepared_msg).await?;

    let maybe_hash = transactions::send_tx(
        api,
        signer.address().clone(),
        &signature,
        &prepared,
        TxStatus::InBlock,
    )
    .await
    .map_err(|e| JsValue::from(e.to_string()))?;

    serialize_to_value(&maybe_hash).map_err(|e| JsValue::from(e.to_string()))
}

#[wasm_bindgen(js_name = "createGuildEncodeParams")]
pub async fn create_guild_encode_params(guild: JsValue) -> Result<JsValue, JsValue> {
    let guild: Guild = deserialize_from_value(guild).map_err(|e| JsValue::from(e.to_string()))?;

    let mut roles = Vec::new();
    for role in guild.roles.into_iter() {
        let ser_requirements =
            cbor_serialize(&role.reqs).map_err(|e| JsValue::from(e.to_string()))?;
        roles.push((role.name, ser_requirements));
    }

    serialize_to_value(&(guild.name, guild.metadata, roles))
        .map_err(|e| JsValue::from(e.to_string()))
}

#[cfg(test)]
mod test {
    use super::*;
    use gn_test_data::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn init_tracing() {
        console_error_panic_hook::set_once();
        tracing_wasm::set_as_global_default();
    }

    #[wasm_bindgen_test]
    async fn test_query_chain() {
        init_tracing();

        let api = Api::from_url(URL).await.unwrap();

        let chain = api.rpc().system_chain().await.unwrap();

        assert_eq!(chain, "Development");
    }

    // NOTE these only work after the guild/join example
    // was successfully run
    #[cfg(feature = "queries")]
    mod queries {
        use super::*;
        use gn_client::{Keypair, Signer, TraitPair};
        use gn_common::identities::Identity;
        use serde_wasm_bindgen::from_value as deserialize_from_value;

        #[wasm_bindgen_test]
        async fn test_query_members() {
            let guild = "myguild".to_string();
            let role = "".to_string();

            let members_js = query_members(guild, role, URL.to_string()).await.unwrap();
            let members_vec: Vec<gn_client::AccountId> =
                deserialize_from_value(members_js).unwrap();

            assert_eq!(members_vec.len(), 6);
        }

        #[wasm_bindgen_test]
        async fn test_query_guilds() {
            let guild_name = "".to_string();
            let guilds = query_guilds(guild_name, URL.to_string()).await.unwrap();
            let guilds_vec: Vec<gn_client::data::GuildData> =
                deserialize_from_value(guilds).unwrap();

            assert_eq!(guilds_vec.len() >= 2);
            for guild in &guilds_vec {
                if guild.name == "myguild" || guild.name == "mysecondguild" {
                    assert_eq!(guild.roles[0], "myrole");
                    assert_eq!(guild.roles[1], "mysecondrole");
                } else {
                    assert_eq!(guild.name == "yellow-guild");
                    assert_eq!(guild.roles[0], "canary-role");
                }
            }
        }

        #[wasm_bindgen_test]
        async fn test_query_requirements() {
            let guild_name = "myguild".to_string();
            let role_name = "myrole".to_string();
            let requirements_js = query_requirements(guild_name, role_name, URL.to_string())
                .await
                .unwrap();
            let requirements: gn_common::requirements::RequirementsWithLogic =
                deserialize_from_value(requirements_js).unwrap();

            assert_eq!(requirements.logic, "0");
        }

        #[wasm_bindgen_test]
        async fn test_query_user_identity() {
            let signer = Signer::new(Keypair::from_seed(&ACCOUNT_SEED));

            let address_string = signer.account_id().to_string();
            let converted_id = AccountId::from_str(&address_string).unwrap();
            assert_eq!(&converted_id, signer.account_id());

            let members_js = query_members("".to_string(), "".to_string(), URL.to_string())
                .await
                .unwrap();
            let members_vec: Vec<AccountId> = deserialize_from_value(members_js).unwrap();
            assert!(members_vec.contains(signer.account_id()));

            let identities_js = query_user_identity(address_string, URL.to_string())
                .await
                .unwrap();

            let identities: Vec<Identity> = deserialize_from_value(identities_js).unwrap();

            assert_eq!(identities.len(), 2);
            match identities[1] {
                Identity::Discord(id) => assert!(id < 10),
                _ => panic!("identity mismatch"),
            }
        }
    }
}
