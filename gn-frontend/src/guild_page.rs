use yew::prelude::*;
use gn_wasm::query_guilds;
use wasm_bindgen_futures::spawn_local;
use gloo_console::log;
use wasm_bindgen::{JsValue};
use js_sys::{Array, Reflect, JsString};


#[derive(Properties, PartialEq)]
pub struct Props {
    pub name: String,
}

#[function_component(GuildPage)]
pub fn guild_page(props: &Props) -> Html {
    let guild = use_state(|| { JsValue::NULL });

    {
        let guild = guild.clone();
        let guild_name = props.name.clone();

        use_effect(move || {
            spawn_local(async move {
                let res: Array = query_guilds(guild_name, String::from("ws://127.0.0.1:9944"))
                  .await
                  .expect("Failed to query guilds")
                  .into();
                
                let res = res.pop();

                guild.set(res);
            });
            
            || {}
        });
    }

    if *guild == JsValue::NULL {
      return html! {
        <h1>{ "Loading..." }</h1>
      }
    }

    let name: JsString = Reflect::get(&guild, &"name".into())
      .expect("Failed to read guild name")
      .into();

    let owner: JsString = Reflect::get(&guild, &"owner".into())
      .expect("Failed to read guild name")
      .into();


    html! {
        <>  
            <h1>{ name }</h1>
            <h2 class="monospace">{ owner }</h2>
        </>
    }
}
