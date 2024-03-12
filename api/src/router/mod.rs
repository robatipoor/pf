use crate::{error::result::ApiResult, handler, server::ApiState};
use anyhow::anyhow;
use axum::{
  extract::DefaultBodyLimit,
  http::HeaderValue,
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
      .layer(cors_layer(&state.config.server.domain_name)?)
      .with_state(state),
  )
}

fn cors_layer(domain_name: &str) -> ApiResult<tower_http::cors::CorsLayer> {
  Ok(
    tower_http::cors::CorsLayer::new()
      .allow_methods([
        hyper::Method::GET,
        hyper::Method::POST,
        hyper::Method::DELETE,
      ])
      .allow_origin(tower_http::cors::AllowOrigin::exact(
        HeaderValue::from_str(domain_name)
          .map_err(|e| anyhow!("Invalid domain url, Error: {e}"))?,
      ))
      .allow_headers([hyper::header::CONTENT_TYPE]),
  )
}
