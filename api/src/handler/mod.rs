pub mod file;

use axum::Json;
use sdk::model::response::MessageResponse;

pub async fn health_check() -> Json<MessageResponse> {
  Json(MessageResponse {
    message: "OK".to_string(),
  })
}
