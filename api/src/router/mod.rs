use crate::{handler, server::ApiState};
use axum::{
  routing::{delete, get, post},
  Router,
};

pub fn get_router(state: ApiState) -> Router {
  Router::new()
    .route("/health_check", get(handler::health_check))
    .route("/:filename", post(handler::upload))
    .route("/:code/:filename", get(handler::download))
    .route("/:code/:filename", delete(handler::delete))
    .with_state(state)
}
