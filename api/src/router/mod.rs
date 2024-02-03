use crate::{handler, server::ApiState};
use axum::{
  extract::DefaultBodyLimit,
  routing::{delete, get, post},
  Router,
};

pub fn get_router(state: ApiState) -> Router {
  Router::new()
    .route("/healthz", get(handler::health_check))
    .route("/upload", post(handler::file::upload))
    .route("/info/:code/:file_name", get(handler::file::info))
    .route("/:code/:file_name", get(handler::file::download))
    .route("/:code/:file_name", delete(handler::file::delete))
    .layer(DefaultBodyLimit::max(state.config.max_upload_bytes_size))
    .with_state(state)
}
