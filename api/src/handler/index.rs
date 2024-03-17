use crate::{error::result::ApiResult, server::ApiState};
use askama::Template;
use axum::{extract::State, response::Html};

#[derive(askama::Template)]
#[template(path = "index.html")]
pub struct IndexPage<'a> {
  pub domain: &'a str,
}

pub async fn page(State(state): State<ApiState>) -> ApiResult<Html<String>> {
  let page = IndexPage {
    domain: &state.config.server.get_domain_name(),
  };
  Ok(Html(page.render()?))
}
