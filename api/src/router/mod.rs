use crate::{handler, server::ApiState};
use axum::{
  extract::DefaultBodyLimit,
  routing::{delete, get, post},
  Router,
};

pub fn get_router(state: ApiState) -> Router {
  Router::new()
    .route("/upload", post(handler::file::upload))
    .layer(DefaultBodyLimit::disable())
    .route("/healthz", get(handler::health_check))
    .route("/info/:code/:file_name", get(handler::file::info))
    .route("/:code/:file_name", get(handler::file::download))
    .route("/:code/:file_name", delete(handler::file::delete))
    .layer(cors_layer())
    .with_state(state)
}

fn cors_layer() -> tower_http::cors::CorsLayer {
  tower_http::cors::CorsLayer::new()
    .allow_methods([
      hyper::Method::GET,
      hyper::Method::POST,
      hyper::Method::DELETE,
    ])
    .allow_origin(tower_http::cors::AllowOrigin::any())
    .allow_headers([hyper::header::CONTENT_TYPE])
}
