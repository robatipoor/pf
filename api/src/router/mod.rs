use crate::{configure::cors::cors_layer, error::result::ApiResult, handler, server::ApiState};
use axum::{
  extract::DefaultBodyLimit,
  routing::{delete, get, post},
  Router,
};

pub fn get_router(state: ApiState) -> ApiResult<Router> {
  Ok(
    Router::new()
      .route("/upload", post(handler::file::upload))
      .layer(DefaultBodyLimit::disable())
      .route("/healthz", get(handler::health_check))
      .route("/info/:code/:file_name", get(handler::file::info))
      .route("/:code/:file_name", get(handler::file::download))
      .route("/:code/:file_name", delete(handler::file::delete))
      .route("/", get(handler::index::page))
      .layer(cors_layer(&state.config.server)?)
      .with_state(state),
  )
}
