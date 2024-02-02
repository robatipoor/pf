pub mod file;

use axum::Json;
use sdk::dto::response::MessageResponse;

pub async fn health_check() -> Json<MessageResponse> {
  Json(MessageResponse::ok())
}
