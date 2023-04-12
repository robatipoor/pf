use axum::{
  body::boxed,
  extract::{BodyStream, Path, Query, State},
  response::Response,
  Json,
};
use common::{
  error::ApiResult,
  model::{
    request::UploadParamQuery,
    response::{MessageResponse, MetaDataFileResponse, UploadResponse},
  },
};
use validator::Validate;

use crate::{server::ApiState, service};

pub async fn upload(
  State(state): State<ApiState>,
  Path(file_name): Path<String>,
  Query(query): Query<UploadParamQuery>,
  body: BodyStream,
) -> ApiResult<Json<UploadResponse>> {
  query.validate()?;
  let pass = None;
  let path = service::file::store(&state, &file_name, &query, pass).await?;
  let url = format!("{}/{path}", state.config.server.get_http_addr());
  todo!()
}

pub async fn download(
  State(state): State<ApiState>,
  Path((code, file_name)): Path<(String, String)>,
) -> ApiResult<Response> {
  let pass = None;
  let meta = service::file::fetch(&state, &code, &file_name, pass).await?;
  // let response = Response::builder().body(boxed(body)).unwrap();
  // Ok(response)
  todo!()
}

pub async fn info(
  State(state): State<ApiState>,
  Path((code, file_name)): Path<(String, String)>,
) -> ApiResult<Json<MetaDataFileResponse>> {
  let meta = service::file::info(&state, &code, &file_name).await?;
  Ok(Json(MetaDataFileResponse::from(&meta)))
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
