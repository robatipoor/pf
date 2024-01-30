use axum::{
  body::Body,
  extract::{Multipart, Path, Query, State},
  http::{header::HeaderMap, Request},
  response::Response,
  Json,
};

use sdk::dto::{
  request::UploadQueryParam,
  response::{MessageResponse, MetaDataFileResponse, UploadResponse},
};

use garde::Validate;
use tower::ServiceExt;
use tower_http::services::fs::ServeFileSystemResponseBody;

use crate::{
  error::ApiResult,
  server::ApiState,
  service::{self},
};

pub async fn upload(
  State(state): State<ApiState>,
  Query(query): Query<UploadQueryParam>,
  headers: HeaderMap,
  multipart: Multipart,
) -> ApiResult<Json<UploadResponse>> {
  query.validate(&())?;
  let secret = crate::util::http::parse_basic_auth(&headers)?;
  let (path, expire_time) = service::file::store(&state, &query, secret, multipart).await?;
  let url = path.url(&state.config.server.get_domain());
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
  let secret = crate::util::http::parse_basic_auth(req.headers())?;
  let file = service::file::fetch(&state, &code, &file_name, secret).await?;
  Ok(file.oneshot(req).await.unwrap())
}

pub async fn info(
  State(state): State<ApiState>,
  Path((code, file_name)): Path<(String, String)>,
  headers: HeaderMap,
) -> ApiResult<Json<MetaDataFileResponse>> {
  let secret = crate::util::http::parse_basic_auth(&headers)?;
  let meta = service::file::info(&state, &code, &file_name, secret).await?;
  Ok(Json(MetaDataFileResponse::from(&meta)))
}

pub async fn delete(
  State(state): State<ApiState>,
  Path((code, file_name)): Path<(String, String)>,
  headers: HeaderMap,
) -> ApiResult<Json<MessageResponse>> {
  let secret = crate::util::http::parse_basic_auth(&headers)?;
  service::file::delete(&state, &code, &file_name, secret).await?;
  Ok(Json(MessageResponse {
    message: "OK".to_string(),
  }))
}
