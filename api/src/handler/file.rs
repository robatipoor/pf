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
  todo!()
}

pub async fn download(
  State(state): State<ApiState>,
  Path((code, file_name)): Path<(String, String)>,
) -> ApiResult<Response> {
  let body = service::get(&code, &file_name).await?;
  let response = Response::builder().body(boxed(body)).unwrap();
  Ok(response)
}

pub async fn delete(
  State(state): State<ApiState>,
  Path((code, filename)): Path<(String, String)>,
) -> ApiResult<Json<MessageResponse>> {
  todo!()
}
