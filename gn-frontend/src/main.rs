mod app;
mod not_found;
mod guild_page;

use app::App;
use not_found::NotFound;
use guild_page::GuildPage;

use yew_router::prelude::*;
use yew::prelude::*;

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
