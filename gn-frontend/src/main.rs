mod app;
mod guild_page;
mod not_found;
mod use_ens_account;

use app::App;
use guild_page::GuildPage;
use not_found::NotFound;

use serde_wasm_bindgen::to_value as serialize_to_value;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/:guild_name")]
    GuildPage { guild_name: String },
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <App /> },
        Route::GuildPage { guild_name } => html! { <GuildPage name={guild_name} /> },
        Route::NotFound => html! { <NotFound /> },
    }
}

#[function_component(Router)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn main() {
    yew::Renderer::<Router>::new().render();
}
