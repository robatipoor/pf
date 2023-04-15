use crate::{handler, server::ApiState};
use axum::{
  extract::DefaultBodyLimit,
  routing::{delete, get, post},
  Router,
};

pub fn get_router(state: ApiState) -> Router {
  Router::new()
    .route("/healthz", get(handler::health_check))
    .route("/web", get(handler::home_page))
    .route("/upload/:filename", post(handler::file::upload))
    .route("/info/:code/:filename", get(handler::file::info))
    .route("/:code/:filename", get(handler::file::download))
    .route("/:code/:filename", delete(handler::file::delete))
    .layer(DefaultBodyLimit::disable())
    .layer(DefaultBodyLimit::max(1024))
    .with_state(state)
}
