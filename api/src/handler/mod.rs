use axum::Json;
use pf_sdk::dto::response::MessageResponse;

pub mod file;
pub mod index;

pub async fn health_check() -> Json<MessageResponse> {
  Json(MessageResponse::ok())
}
