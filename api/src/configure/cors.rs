use crate::{configure::ServerConfig, error::result::ApiResult};
use anyhow::anyhow;
use axum::http::HeaderValue;

pub fn cors_layer(config: &ServerConfig) -> ApiResult<tower_http::cors::CorsLayer> {
  let domain = HeaderValue::from_str(&config.domain_name)
    .map_err(|err| anyhow!("Invalid domain url, Error: {err}"))?;
  let allow_origin = if let Some(ref public_addr) = config.get_public_addr() {
    let public_addr = HeaderValue::from_str(public_addr)
      .map_err(|err| anyhow!("Invalid public_addr, Error: {err}"))?;
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
