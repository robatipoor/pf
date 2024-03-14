use crate::{configure::ServerConfig, error::result::ApiResult, handler, server::ApiState};
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
      .layer(cors_layer(&state.config.server)?)
      .with_state(state),
  )
}

fn cors_layer(config: &ServerConfig) -> ApiResult<tower_http::cors::CorsLayer> {
  let domain = HeaderValue::from_str(&config.domain_name)
    .map_err(|e| anyhow!("Invalid domain url, Error: {e}"))?;
  let allow_origin = if let Some(ref public_addr) = config.public_addr {
    let public_addr =
      HeaderValue::from_str(public_addr).map_err(|e| anyhow!("Invalid public_addr, Error: {e}"))?;
    tower_http::cors::AllowOrigin::list(vec![public_addr, domain])
  } else {
    tower_http::cors::AllowOrigin::exact(domain)
  };
  Ok(
    tower_http::cors::CorsLayer::new()
      .allow_methods([
        hyper::Method::GET,
        hyper::Method::POST,
        hyper::Method::DELETE,
      ])
      .allow_origin(allow_origin)
      .allow_headers([hyper::header::CONTENT_TYPE]),
  )
}
