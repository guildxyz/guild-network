use yew::prelude::*;
use gn_wasm::query_guilds;
use wasm_bindgen_futures::spawn_local;
use gloo_console::log;
use js_sys::{Array, Reflect, JsString};

#[function_component(NotFound)]
pub fn not_found() -> Html {
    html! {
        <>  
            <h1>{"Guild Network - 404"}</h1>
        </>
    }
}
