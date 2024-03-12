use axum::{
  body::Body,
  extract::{Multipart, Path, Query, State},
  http::{header::HeaderMap, Request},
  response::Response,
  Json,
};
use garde::Validate;
use sdk::{
  dto::{
    request::UploadQueryParam,
    response::{MessageResponse, MetaDataFileResponse, UploadResponse},
  },
  util::url::create_url,
};
use tower::ServiceExt;
use tower_http::services::fs::ServeFileSystemResponseBody;

use crate::{error::result::ApiResult, server::ApiState, service, util::qr_code::generate_qr_code};

pub async fn upload(
  State(state): State<ApiState>,
  Query(param): Query<UploadQueryParam>,
  headers: HeaderMap,
  multipart: Multipart,
) -> ApiResult<Json<UploadResponse>> {
  param.validate(&())?;
  let secret = crate::util::http::parse_basic_auth(&headers)?;
  let (file_path, expire_date_time) =
    service::file::store(&state, &param, secret, multipart).await?;
  let url = create_url(
    &state.config.server.domain_name,
    &file_path.code,
    &file_path.file_name,
  )?
  .to_string();
  let qr_code = if let Some(qr_code_format) = param.qr_code_format {
    Some(generate_qr_code(qr_code_format, &url)?)
  } else {
    None
  };
  Ok(Json(UploadResponse {
    url,
    expire_date_time,
    qr_code,
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
  Ok(Json(MessageResponse::ok()))
}
