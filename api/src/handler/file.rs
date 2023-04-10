use axum::{
  body::boxed,
  extract::{BodyStream, Path, State},
  response::Response,
  Json,
};
use common::{error::ApiResult, model::response::MessageResponse};

use crate::{server::ApiState, service};

pub async fn upload(
  State(state): State<ApiState>,
  Path(file_name): Path<String>,
  body: BodyStream,
) -> ApiResult<Response> {
  let code = service::file::store(&state, &file_name);
  todo!()
}

pub async fn download(
  State(state): State<ApiState>,
  Path((code, file_name)): Path<(String, String)>,
) -> ApiResult<Response> {
  service::file::fetch(&state, &code, &file_name).await?;
  // let response = Response::builder().body(boxed(body)).unwrap();
  // Ok(response)
  todo!()
}

pub async fn info(
  State(state): State<ApiState>,
  Path((code, file_name)): Path<(String, String)>,
) -> ApiResult<Response> {
  service::file::info(&state, &code, &file_name).await?;
  // let response = Response::builder().body(boxed(body)).unwrap();
  // Ok(response)
  todo!()
}

pub async fn delete(
  State(state): State<ApiState>,
  Path((code, file_name)): Path<(String, String)>,
) -> ApiResult<Json<MessageResponse>> {
  service::file::delete(&state, &code, &file_name).await?;
  Ok(Json(MessageResponse {
    message: "OK".to_string(),
  }))
}
