use axum::{
  response::{IntoResponse, Response},
  Json,
};

use super::ApiError;

impl IntoResponse for ApiError {
  fn into_response(self) -> Response {
    let (body, status_code) = self.response();
    (status_code, Json(body)).into_response()
  }
}
