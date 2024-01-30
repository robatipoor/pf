use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::{footer::Footer, header::Header};

#[derive(Routable, Debug, Clone, PartialEq, Eq)]
pub enum AppRoute {
  #[at("/")]
  Home,
  #[not_found]
  #[at("/404")]
  NotFound,
}

pub fn switch(_route: AppRoute) -> Html {
  html! { "Page not found" }
}

#[function_component(App)]
pub fn app() -> Html {
  html! {
      <HashRouter>
               <Header />
              <Switch<AppRoute> render={switch} />
              <Footer />
       </HashRouter>
  }
}