use axum::extract::BodyStream;
use axum_extra::body::AsyncReadBody;
use chrono::Utc;
use common::error::{ApiError, ApiResult};
use common::model::request::UploadParamQuery;
use futures_util::TryStreamExt;
use tokio::fs::File;
use tokio::io::BufWriter;
use tokio::time::Instant;
use tokio_util::io::StreamReader;

use crate::database::MetaDataFile;
use crate::server::ApiState;

pub async fn store(
  state: &ApiState,
  file_name: &str,
  query: &UploadParamQuery,
  password: Option<String>,
) -> ApiResult<String> {
  let hash_password = password.map(|p| p.to_lowercase());
  let code = loop {
    let code: String = common::util::string::generate_random_string(query.length_code.unwrap());
    let path = format!("{code}/{file_name}");
    if !state.db.exist(&path).await {
      let meta = MetaDataFile {
        create_at: Utc::now(),
        expire_time: Utc::now(),
        is_deleteable: query.deleteable.unwrap(),
        max_download: query.max_download,
        password: hash_password,
        downloads: 0,
      };
      state.db.store(path, meta).await?;
      break code;
    }
  };
  Ok(code)
}

pub async fn info(state: &ApiState, code: &str, file_name: &str) -> ApiResult<MetaDataFile> {
  let path = format!("{code}/{file_name}");
  state
    .db
    .fetch(&path)
    .await
    .ok_or_else(|| ApiError::NotFound(format!("{path} not found")))
}

pub async fn fetch(
  state: &ApiState,
  code: &str,
  file_name: &str,
  password: Option<String>,
) -> ApiResult<MetaDataFile> {
  let path = format!("{code}/{file_name}");
  state
    .db
    .fetch_count(&path)
    .await
    .filter(|m| {
      m.expire_time < Utc::now()
        && (m.max_download.is_none() || m.max_download.unwrap() >= m.downloads)
        && (m.password.is_none() // TODO hash func
          || (password.is_some() && &password.unwrap() == m.password.as_ref().unwrap()))
    })
    .ok_or_else(|| ApiError::NotFound(format!("{path} not found")))
}

pub async fn delete(state: &ApiState, code: &str, file_name: &str) -> ApiResult<()> {
  let path = format!("{code}/{file_name}");
  if let Some(info) = state.db.fetch(&path).await {
    if info.is_deleteable {
      state.db.delete(&path).await;
    } else {
      return Err(ApiError::PermissionDenied(format!(
        "it is not possible to delete {path}"
      )));
    }
  }
  Ok(())
}
