use yew::prelude::*;
use gn_wasm::query_guilds;
use wasm_bindgen_futures::spawn_local;
use gloo_console::log;
use js_sys::{Array, Reflect, JsString};

#[function_component(App)]
pub fn app() -> Html {
    let guilds = use_state(|| { Array::new() });

    {
        let guilds = guilds.clone();

        use_effect(move || {
            spawn_local(async move {
                guilds.set(
                    query_guilds(String::from(""), String::from("ws://127.0.0.1:9944"))
                        .await
                        .expect("Failed to query guilds")
                        .into()
                );
            });
            
            || {}
        });
    }


    html! {
        <>  
            <h1>{"Guild Network"}</h1>
            {
                guilds.iter().map(|g| {
                    let guild_name: JsString = Reflect::get(&g, &"name".into())
                        .expect("Failed to access guild \"name\" property")
                        .into();

                    html! {
                        <div>{ &guild_name }</div>
                    }
                })
                .collect::<Html>()
            }
        </>
    }
}
