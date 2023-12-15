use axum::{
  body::Body,
  extract::{multipart::Field, Multipart, Path, Query, State},
  http::{header::HeaderMap, Request},
  response::Response,
  Json,
};
use futures_util::{Stream, TryStreamExt};
use sdk::model::{
  request::UploadParamQuery,
  response::{MessageResponse, MetaDataFileResponse, UploadResponse},
};
use tokio_util::{bytes::Bytes, io::StreamReader};
use tower::ServiceExt;
use tower_http::services::fs::ServeFileSystemResponseBody;
use validator::Validate;

use crate::{
  error::ApiResult,
  server::ApiState,
  service::{self, file},
};

pub async fn upload(
  State(state): State<ApiState>,
  Path(file_name): Path<String>,
  Query(query): Query<UploadParamQuery>,
  headers: HeaderMap,
  multipart: Multipart,
) -> ApiResult<Json<UploadResponse>> {
  crate::util::file_name::validate(&file_name)?;
  query.validate()?;
  let auth = crate::util::http::parse_basic_auth(&headers)?;
  let (path, expire_time) =
    service::file::store(&state, &file_name, &query, auth, multipart).await?;
  let url = format!("{}/{path}", state.config.domain);
  let qrcode = crate::util::qrcode::encode(&url)?;
  Ok(Json(UploadResponse {
    url,
    expire_time,
    qrcode,
  }))
}

pub async fn download(
  State(state): State<ApiState>,
  Path((code, file_name)): Path<(String, String)>,
  req: Request<Body>,
) -> ApiResult<Response<ServeFileSystemResponseBody>> {
  let auth = crate::util::http::parse_basic_auth(req.headers())?;
  let file = service::file::fetch(&state, &code, &file_name, auth).await?;
  Ok(file.oneshot(req).await.unwrap())
}

pub async fn info(
  State(state): State<ApiState>,
  Path((code, file_name)): Path<(String, String)>,
  headers: HeaderMap,
) -> ApiResult<Json<MetaDataFileResponse>> {
  let auth = crate::util::http::parse_basic_auth(&headers)?;
  let meta = service::file::info(&state, &code, &file_name, auth).await?;
  Ok(Json(MetaDataFileResponse::from(&meta)))
}

pub async fn delete(
  State(state): State<ApiState>,
  Path((code, file_name)): Path<(String, String)>,
  headers: HeaderMap,
) -> ApiResult<Json<MessageResponse>> {
  let auth = crate::util::http::parse_basic_auth(&headers)?;
  service::file::delete(&state, &code, &file_name, auth).await?;
  Ok(Json(MessageResponse {
    message: "OK".to_string(),
  }))
}