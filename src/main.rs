#![recursion_limit = "2048"]
#[macro_use]
extern crate lazy_static;
use gloo_utils::document;
use pages::home::Home;
use yew::{prelude::*, start_app_in_element};
use yew_router::prelude::*;

mod components;
mod fetch;
mod pages;

lazy_static! {
    pub static ref BASE_URL: String =
        "http://localhost:8080/content/".to_string();
}

#[derive(Routable, Clone, PartialEq)]
enum WebRoute {
    #[not_found]
    #[at("/404")]
    PageNotFound,
    #[at("/")]
    Home,
}

struct Website;

impl Component for Website {
    type Message = ();
    type Properties = ();
    fn create(_: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, _ctx: &Context<Self>, _: Self::Message) -> bool {
        true
    }

    fn view(&self, _: &Context<Self>) -> Html {
        html! {
            <BrowserRouter>
                <main>
                    <Switch<WebRoute>
                        render={Switch::render(Website::switch)}
                    />
                </main>
            </BrowserRouter>
        }
    }
}

impl Website {
    fn switch(route: &WebRoute) -> Html {
        match route {
            WebRoute::Home => html! {<Home/>},
            WebRoute::PageNotFound => html! {"404 Not found"},
        }
    }
}

pub fn main() {
    let document = document();
    let body = document.query_selector("#mount").unwrap().unwrap();

    start_app_in_element::<Website>(body);
}
