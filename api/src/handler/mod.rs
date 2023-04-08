use axum::{
  extract::{Multipart, Path, State},
  response::Html,
  Json,
};
use common::{error::ApiResult, model::response::MessageResponse};

use crate::server::ApiState;

pub async fn health_check() -> MessageResponse {
  MessageResponse {
    message: "OK".to_string(),
  }
}

pub async fn home_page() -> Html<&'static str> {
  Html::from("")
}

pub async fn upload(
  State(state): State<ApiState>,
  mut multipart: Multipart,
) -> ApiResult<Json<MessageResponse>> {
  while let Some(field) = multipart.next_field().await.unwrap() {
    let name = field.name().unwrap().to_string();
    let file_name = field.file_name().unwrap().to_string();
    let content_type = field.content_type().unwrap().to_string();
    let data = field.bytes().await.unwrap();
  }
  todo!()
}

pub async fn download(
  State(state): State<ApiState>,
  Path((code, filename)): Path<(String, String)>,
) -> ApiResult<Json<MessageResponse>> {
  todo!()
}

pub async fn delete(
  State(state): State<ApiState>,
  Path((code, filename)): Path<(String, String)>,
) -> ApiResult<Json<MessageResponse>> {
  todo!()
}
