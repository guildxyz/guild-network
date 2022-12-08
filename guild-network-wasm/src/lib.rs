use guild_network_client::queries::{self, GuildFilter};
use guild_network_client::Api;
use guild_network_common::{pad::pad_to_32_bytes, GuildName};
use wasm_bindgen::prelude::*;
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
    let members_vec: Vec<guild_network_client::AccountId> = members_js.into_serde().unwrap();

    assert_eq!(members_vec.len(), 6);
}

#[wasm_bindgen_test]
async fn test_query_guilds() {
    let guild_name = "".to_string();
    let guilds = query_guilds(guild_name, URL.to_string()).await.unwrap();
    let guilds_vec: Vec<guild_network_client::data::GuildData> = guilds.into_serde().unwrap();

    assert_eq!(guilds_vec.len(), 2);
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
