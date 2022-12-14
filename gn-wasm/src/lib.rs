use gn_client::queries::{self, GuildFilter};
use gn_client::transactions::{self, TxStatus};
use gn_client::{
    AccountId, Api, RuntimeIdentityWithAuth, Signature, SrSignature, SubstrateAddress,
};
use gn_common::{pad::pad_to_32_bytes, utils, GuildName};
use serde_wasm_bindgen::to_value as serialize_to_value;
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
    utils::verification_msg(&address)
}

#[wasm_bindgen(js_name = "registerTxPayload")]
pub async fn register_tx_payload(
    address: String,
    evm_address: String,
    evm_signature: String,
    discord: Option<String>,
    telegram: Option<String>,
    url: String,
) -> Result<JsValue, JsValue> {
    let api = Api::from_url(&url)
        .await
        .map_err(|e| JsValue::from(e.to_string()))?;

    let account_id = AccountId::from_str(&address).map_err(|e| JsValue::from(e.to_string()))?;
    let mut evm_address_bytes = [0u8; 20];
    let mut evm_signature_bytes = [0u8; 65];

    hex::decode_to_slice(&evm_address, &mut evm_address_bytes)
        .map_err(|e| JsValue::from(e.to_string()))?;
    hex::decode_to_slice(&evm_signature, &mut evm_signature_bytes)
        .map_err(|e| JsValue::from(e.to_string()))?;

    let mut identities = vec![RuntimeIdentityWithAuth::EvmChain(
        evm_address_bytes,
        evm_signature_bytes,
    )];

    if let Some(dc_id) = discord {
        let dc_id_u64 = dc_id
            .parse::<u64>()
            .map_err(|e| JsValue::from(e.to_string()))?;
        identities.push(RuntimeIdentityWithAuth::Discord(dc_id_u64, ()));
    }

    if let Some(tg_id) = telegram {
        let tg_id_u64 = tg_id
            .parse::<u64>()
            .map_err(|e| JsValue::from(e.to_string()))?;
        identities.push(RuntimeIdentityWithAuth::Telegram(tg_id_u64, ()));
    }

    let tx_payload = transactions::register(identities);

    let prepared = api
        .tx()
        .prepare_unsigned(&tx_payload, &account_id, Default::default())
        .await
        .map_err(|e| JsValue::from(e.to_string()))?;

    serialize_to_value(&prepared).map_err(|e| JsValue::from(e.to_string()))
}

#[wasm_bindgen(js_name = "joinGuildTxPayload")]
pub async fn join_guild_tx_payload(
    address: String,
    guild: String,
    role: String,
    url: String,
) -> Result<JsValue, JsValue> {
    if guild.len() > 32 || role.len() > 32 {
        return Err(JsValue::from("too long input length"));
    }

    let api = Api::from_url(&url)
        .await
        .map_err(|e| JsValue::from(e.to_string()))?;
    let account_id = AccountId::from_str(&address).map_err(|e| JsValue::from(e.to_string()))?;

    let tx_payload = transactions::join_guild(pad_to_32_bytes(&guild), pad_to_32_bytes(&role));

    let prepared = api
        .tx()
        .prepare_unsigned(&tx_payload, &account_id, Default::default())
        .await
        .map_err(|e| JsValue::from(e.to_string()))?;

    serialize_to_value(&prepared).map_err(|e| JsValue::from(e.to_string()))
}

#[wasm_bindgen(js_name = "sendTransaction")]
pub async fn send_transaction(
    address: String,
    signature: String,
    encoded_params: Vec<u8>,
    url: String,
) -> Result<JsValue, JsValue> {
    let api = Api::from_url(&url)
        .await
        .map_err(|e| JsValue::from(e.to_string()))?;
    let account_id = AccountId::from_str(&address).map_err(|e| JsValue::from(e.to_string()))?;
    let substrate_address = SubstrateAddress::from(account_id);

    let mut signature_bytes = [0u8; 64];
    hex::decode_to_slice(&signature, &mut signature_bytes)
        .map_err(|e| JsValue::from(e.to_string()))?;
    let signature = Signature::Sr25519(SrSignature::from_raw(signature_bytes));

    let mut progress = api
        .tx()
        .pack_and_submit_then_watch(substrate_address, signature, &encoded_params)
        .await
        .map_err(|e| JsValue::from(e.to_string()))?;

    let maybe_hash = transactions::track_progress(&mut progress, TxStatus::InBlock)
        .await
        .map_err(|e| JsValue::from(e.to_string()))?;

    serialize_to_value(&maybe_hash).map_err(|e| JsValue::from(e.to_string()))
}

#[cfg(test)]
mod test {
    use super::*;
    use gn_client::{AccountKeyring, Keypair, Signer, TraitPair};
    use gn_common::identities::Identity;
    use serde_wasm_bindgen::from_value as deserialize_from_value;
    use wasm_bindgen_test::*;
    const URL: &str = "ws://127.0.0.1:9944";

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

    #[wasm_bindgen_test]
    async fn test_query_members() {
        let guild = "myguild".to_string();
        let role = "".to_string();

        let members_js = query_members(guild, role, URL.to_string()).await.unwrap();
        let members_vec: Vec<gn_client::AccountId> = deserialize_from_value(members_js).unwrap();

        assert_eq!(members_vec.len(), 6);
    }

    #[wasm_bindgen_test]
    async fn test_query_guilds() {
        let guild_name = "".to_string();
        let guilds = query_guilds(guild_name, URL.to_string()).await.unwrap();
        let guilds_vec: Vec<gn_client::data::GuildData> = deserialize_from_value(guilds).unwrap();

        assert_eq!(guilds_vec.len(), 2);
        assert_eq!(guilds_vec[0].roles.len(), 2);
        assert_eq!(guilds_vec[1].roles.len(), 2);
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
        let seed = [10u8; 32];
        let signer = Signer::new(Keypair::from_seed(&seed));

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

    #[wasm_bindgen_test]
    async fn test_tx_submission() {
        // assuming that the guild/join test has been successfully
        // completed this signer should be able to join a specific guild
        let seed = [10u8; 32];
        let signer = Arc::new(Signer::new(Keypair::from_seed(&seed)));
        // create api for queries
        let api = Api::from_url(URL).await.unwrap();
        let members = queries::members(api.clone(), Some(&GuildFilter { name:  ,10);

        // join_guild tx payload
        // sign payload msg manually
        // send transaction
        // query members
    }
}
