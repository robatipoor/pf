use axum::Json;
use sdk::dto::response::MessageResponse;

pub mod file;
pub mod index;

pub async fn health_check() -> Json<MessageResponse> {
  Json(MessageResponse::ok())
}
