use super::serialize_to_value;
use gloo_console::log;
use gn_wasm::query_guilds;
use js_sys::{Array, JsString, Reflect};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub name: String,
}

#[function_component(GuildPage)]
pub fn guild_page(props: &Props) -> Html {
    let guild = use_state(|| JsValue::NULL);

    {
        let guild = guild.clone();
        let guild_name = props.name.clone();
        use_effect_with_deps(
            move |_| {
                spawn_local(async move {
                    let res: Array = query_guilds(guild_name, String::from("ws://127.0.0.1:9944"))
                        .await
                        .expect("failed to query guilds")
                        .into();

                    let res = res.pop();

                    guild.set(res);
                });

                || {}
            },
            (),
        );
    }

    if *guild == JsValue::NULL {
        return html! {
          <h1>{ "Loading..." }</h1>
        };
    }

    let name: JsString = Reflect::get(&guild, &"name".into())
        .expect("Failed to read guild name")
        .into();

    let owner: JsString = Reflect::get(&guild, &"owner".into())
        .expect("Failed to read guild name")
        .into();

    let roles: Array = Reflect::get(&guild, &"roles".into())
        .expect("Failed to read guild name")
        .into();

    log!(&*guild);

    html! {
        <>
            <h1>{ name }</h1>
            <h2 class="monospace">{ owner }</h2>
            <div class="vertical-flex-container">
              { roles.iter().map(|role| {
                let role_name: JsString = role.into();
                html! {
                  <span>{ role_name }</span>
                }
              }).collect::<Html>() }
            </div>
        </>
    }
}
