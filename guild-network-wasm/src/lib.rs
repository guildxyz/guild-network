use guild_network_client::data::Guild;
use guild_network_client::transactions;
use guild_network_common::requirements::RequirementWithLogic;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = "createGuild")]
pub async fn create_guild(js_guild: JsValue) -> Result<JsValue, JsValue> {
    let guild: Guild = js_guild
        .into_serde()
        .map_err(|e| JsValue::from(e.to_string()))?;
    let tx = transactions::create_guild(guild).map_err(|e| JsValue::from(e.to_string()))?;
    JsValue::from_serde(tx).map_err(|e| JsValue::from(e.to_string()))
}
