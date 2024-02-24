use axum::{
  body::Body,
  extract::{Multipart, Path, Query, State},
  http::{header::HeaderMap, Request},
  response::Response,
  Json,
};
use garde::Validate;
use sdk::dto::{
  request::{QrCodeFormat, UploadQueryParam},
  response::{MessageResponse, MetaDataFileResponse, UploadResponse},
  FileUrlPath,
};
use tower::ServiceExt;
use tower_http::services::fs::ServeFileSystemResponseBody;

use crate::{
  error::result::ApiResult,
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
  let (file_path, expire_date_time) =
    service::file::store(&state, &query, secret, multipart).await?;
  let url = FileUrlPath::from(file_path)
    .to_url(&state.config.server.get_domain())?
    .to_string();
  let qr_code = generate_qr_code(query.qr_code_format, &url)?;
  Ok(Json(UploadResponse {
    url,
    expire_date_time,
    qr_code,
  }))
}

pub fn generate_qr_code(
  qr_code_format: Option<QrCodeFormat>,
  input: &str,
) -> ApiResult<Option<String>> {
  match qr_code_format {
    Some(QrCodeFormat::Text) => Ok(Some(sdk::util::qr_code::generate_base64_text_qr_code(
      input,
    )?)),
    Some(QrCodeFormat::Image) => Ok(Some(sdk::util::qr_code::generate_base64_png_qr_code(
      input,
    )?)),
    None => Ok(None),
  }
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
