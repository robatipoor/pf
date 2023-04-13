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
) -> ApiResult<String> {
  let hash_password = query
    .password
    .as_ref()
    .map(common::util::hash::argon_hash)
    .transpose()
    .map_err(|e| ApiError::HashError(e.to_string()))?;
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
  let meta = state
    .db
    .fetch(&path)
    .await
    .ok_or_else(|| ApiError::NotFound(format!("{path} not found")))?;
  if let Some(max) = meta.max_download {
    if meta.downloads >= max {
      state.db.delete(path.clone()).await;
      return Err(ApiError::NotFound(format!("{path} not found")));
    }
  }
  Ok(meta)
}

pub async fn fetch(
  state: &ApiState,
  code: &str,
  file_name: &str,
  password: Option<String>,
) -> ApiResult<MetaDataFile> {
  let path = format!("{code}/{file_name}");
  let meta = state
    .db
    .fetch_count(&path)
    .await
    .ok_or_else(|| ApiError::NotFound(format!("{path} not found")))?;
  if let Some(max) = meta.max_download {
    if meta.downloads >= max {
      state.db.delete(path.clone()).await;
      return Err(ApiError::NotFound(format!("{path} not found")));
    }
  }
  if let Some(hash) = meta.password.as_ref() {
    if !matches!(
      password.map(|p| common::util::hash::argon_verify(p, hash)),
      Some(Ok(()))
    ) {
      return Err(ApiError::PermissionDenied("password invalid".to_string()));
    }
  }
  Ok(meta)
}

pub async fn delete(state: &ApiState, code: &str, file_name: &str) -> ApiResult<()> {
  let path = format!("{code}/{file_name}");
  if let Some(info) = state.db.fetch(&path).await {
    if info.is_deleteable {
      state.db.delete(path).await;
    } else {
      return Err(ApiError::PermissionDenied(format!(
        "it is not possible to delete {path}"
      )));
    }
  }
  Ok(())
}
