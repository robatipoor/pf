use axum::{
  body::boxed,
  extract::{BodyStream, Path, Query, State},
  response::Response,
  Json, TypedHeader,
};
use common::{
  error::ApiResult,
  model::{
    request::UploadParamQuery,
    response::{MessageResponse, MetaDataFileResponse, UploadResponse},
  },
};
use hyper::HeaderMap;
use validator::Validate;

use crate::{server::ApiState, service};

pub async fn upload(
  State(state): State<ApiState>,
  Path(file_name): Path<String>,
  Query(query): Query<UploadParamQuery>,
  headers: HeaderMap,
  body: BodyStream,
) -> ApiResult<Json<UploadResponse>> {
  common::util::file_name::validate(&file_name)?;
  query.validate()?;
  let auth = common::util::http::parse_basic_auth(&headers)?;
  let (path, expire_time) = service::file::store(&state, &file_name, &query, auth).await?;
  let url = format!("{}/{path}", state.config.server.domain);
  let qrcode = common::util::qrcode::encode(&url)?;
  Ok(Json(UploadResponse {
    url,
    expire_time,
    qrcode,
  }))
}

pub async fn download(
  State(state): State<ApiState>,
  Path((code, file_name)): Path<(String, String)>,
  headers: HeaderMap,
) -> ApiResult<Response> {
  let auth = common::util::http::parse_basic_auth(&headers)?;
  let meta = service::file::fetch(&state, &code, &file_name, auth).await?;
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
