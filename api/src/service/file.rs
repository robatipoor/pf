use axum::extract::BodyStream;
use axum_extra::body::AsyncReadBody;
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
  loop {
    let code: String = common::util::string::generate_random_string(query.length_code.unwrap());
    let path = format!("{code}/{file_name}");
    if !state.db.exist(&path).await {
      let meta = MetaDataFile {
        create_at: Instant::now(),
        expire_time: Instant::now(),
        is_deleteable: query.deleteable.unwrap(),
        max_download: query.max_download,
        downloads: 0,
      };
      state.db.store(path, meta).await?;
      return Ok(code);
    }
  }
}

pub async fn info(state: &ApiState, code: &str, file_name: &str) -> ApiResult<MetaDataFile> {
  let path = format!("{code}/{file_name}");
  state
    .db
    .fetch_any(&path)
    .await
    .ok_or_else(|| ApiError::NotFound(format!("{path} not found")))
}

pub async fn fetch(state: &ApiState, code: &str, file_name: &str) -> ApiResult<MetaDataFile> {
  let path = format!("{code}/{file_name}");
  state
    .db
    .fetch_count(&path)
    .await
    .ok_or_else(|| ApiError::NotFound(format!("{path} not found")))
}

pub async fn delete(state: &ApiState, code: &str, file_name: &str) -> ApiResult<()> {
  let path = format!("{code}/{file_name}");
  if let Some(info) = state.db.fetch_any(&path).await {
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
