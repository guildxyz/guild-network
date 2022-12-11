use crate::Route;
use gloo_console::log;
use gn_wasm::query_guilds;
use js_sys::{Array, JsString, Reflect};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;
use crate::use_ens_account::use_ens_account;

#[function_component(App)]
pub fn app() -> Html {
    let guilds = use_state(|| Array::new());
    let ens_account = use_ens_account();

    {   
        let ens_account = ens_account.clone();
        let ens_account_dep = ens_account.clone();

        use_effect_with_deps(
            move |_| {
                log!(&*ens_account);

                || {}
            },
            ens_account_dep,
        );
    }

    {
        let guilds = guilds.clone();

        use_effect_with_deps(
            move |_| {
                spawn_local(async move {
                    guilds.set(
                        query_guilds(String::from(""), String::from("ws://127.0.0.1:9944"))
                            .await
                            .expect("Failed to query guilds")
                            .into(),
                    );
                });

                || {}
            },
            (),
        );
    }

    log!(&*guilds);

    html! {
        <>
            <h1>{"Guild Network"}</h1>
            {
                guilds.iter().map(|g| {
                    let guild_name: JsString = Reflect::get(&g, &"name".into())
                        .expect("Failed to access guild \"name\" property")
                        .into();

                    let guild_name_string: String = guild_name.as_string().unwrap();

                    html! {
                        <Link<Route> to={Route::GuildPage { guild_name: guild_name_string }}><div>{ &guild_name }</div></Link<Route>>
                    }
                })
                .collect::<Html>()
            }
            <w3m-core-button></w3m-core-button>
            <w3m-modal></w3m-modal>
        </>
    }
}
