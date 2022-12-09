use gn_client::queries::{self, GuildFilter};
use gn_client::{AccountId, Api, ClientConfig, Index, Signature, SubstrateAddress, TxSignerTrait};
use gn_common::{pad::pad_to_32_bytes, GuildName};
use js_sys::{Error, Function, JsString, Object, Promise};
use wasm_bindgen::prelude::*;
use web_sys::{console, window};

use std::str::FromStr;

fn get_sign_request(msg: &str, address: &str) -> Result<js_sys::Object, Error> {
    let sign_request_param = js_sys::Object::new();

    js_sys::Reflect::set(
        &sign_request_param,
        &JsString::from("address"),
        &address.into(),
    )?;
    js_sys::Reflect::set(&sign_request_param, &JsString::from("data"), &msg.into())?;

    Ok(sign_request_param)
}

pub struct WasmSigner {
    pub lib: Object,
    pub account_id: AccountId,
    pub address: SubstrateAddress,
}

impl WasmSigner {
    pub async fn new() -> Result<Self, Error> {
        let window = window().expect("Failed to access window object");

        let injected_web3 = window
            .get("injectedWeb3")
            .expect("Failed to access window.injectedWeb3");

        let polkadot_js: JsValue =
            js_sys::Reflect::get(&injected_web3, &JsString::from("polkadot-js"))?;

        let enable: Function =
            js_sys::Reflect::get(&polkadot_js, &JsString::from("enable"))?.into();

        let lib: Object =
            wasm_bindgen_futures::JsFuture::from(Promise::from(enable.call0(&JsValue::NULL)?))
                .await?
                .into();

        let addresses: js_sys::Array = wasm_bindgen_futures::JsFuture::from(Promise::from(
            js_sys::Function::from(js_sys::Reflect::get(
                &js_sys::Reflect::get(&lib, &JsString::from("accounts"))?,
                &JsString::from("get"),
            )?)
            .call0(&JsValue::NULL)?,
        ))
        .await?
        .into();

        let name: String = js_sys::Reflect::get(&addresses.at(0), &"name".into())?
            .as_string()
            .expect("Failed to cast addresses[0] to String");
        let address: String = js_sys::Reflect::get(&addresses.at(0), &"address".into())?
            .as_string()
            .expect("Failed to cast addresses[0] to String");

        console::log_1(&format!("Hello {}! ({})", name, address).into());

        let account_id = AccountId::from_str(&address).expect("invalid address");
        let address = SubstrateAddress::from(account_id.clone());

        Ok(Self {
            lib,
            account_id, 
            address,
        })
    }
}

impl TxSignerTrait<ClientConfig> for WasmSigner {
    fn nonce(&self) -> Option<Index> {
        Some(123)
    }

    fn account_id(&self) -> &AccountId {
        &self.account_id
    }

    fn address(&self) -> SubstrateAddress {
        self.address
    }

    fn sign(&self, signer_payload: &[u8]) -> Signature {
        let signer =
            js_sys::Reflect::get(&self.lib, &"signer".into()).expect("failed to get signer");
        let sign_raw: Function = js_sys::Reflect::get(&signer, &"signRaw".into())
            .expect("failed to get signature function")
            .into();

        let sign_payload =
            get_sign_request(signer_payload, &self.address).expect("failed to get sign request");

        sign_raw
            .call1(&JsValue::NULL, &sign_payload)
            .expect("failed to get sign promise")
            .into()
    }
}

#[wasm_bindgen(js_name = "polkadotSign")]
pub async fn polkadot_sign(message: &str) -> Result<(), Error> {
    console::log_2(&"Message to sign:".into(), &message.into());

    let signer = WasmSigner::new().await?;
    let signature = signer.sign(message.as_bytes());

    console::log_1(&signature);
    Ok(())
}

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

    JsValue::from_serde(&members).map_err(|e| JsValue::from(e.to_string()))
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

    JsValue::from_serde(&guilds).map_err(|e| JsValue::from(e.to_string()))
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

        JsValue::from_serde(&requirements).map_err(|e| JsValue::from(e.to_string()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
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
        let members_vec: Vec<gn_client::AccountId> = members_js.into_serde().unwrap();

        assert_eq!(members_vec.len(), 6);
    }

    #[wasm_bindgen_test]
    async fn test_query_guilds() {
        let guild_name = "".to_string();
        let guilds = query_guilds(guild_name, URL.to_string()).await.unwrap();
        let guilds_vec: Vec<gn_client::data::GuildData> = guilds.into_serde().unwrap();

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
            requirements_js.into_serde().unwrap();

        assert_eq!(requirements.logic, "0");
    }
}
