use crate::{handler, server::ApiState};
use axum::{
  extract::DefaultBodyLimit,
  routing::{delete, get, post},
  Router,
};

pub fn get_router(state: ApiState) -> Router {
  Router::new()
    .route("/health_check", get(handler::health_check))
    .route("/", get(handler::home_page))
    .route("/:filename", post(handler::file::upload))
    .route("/:code/:filename", get(handler::file::download))
    .route("/:code/:filename", delete(handler::file::delete))
    .layer(DefaultBodyLimit::disable())
    .layer(DefaultBodyLimit::max(1024))
    .with_state(state)
}
