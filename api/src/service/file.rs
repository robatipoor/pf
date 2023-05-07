use axum::extract::BodyStream;
use chrono::{DateTime, Utc};
use common::error::{ApiError, ApiResult, ToApiResult};
use common::model::request::UploadParamQuery;
use futures_util::TryStreamExt;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::BufWriter;
use tokio_util::io::StreamReader;
use tower_http::services::ServeFile;
use tracing::debug;

use crate::database::{MetaDataFile, PathFile};
use crate::server::ApiState;

pub async fn store(
  state: &ApiState,
  file_name: &str,
  query: &UploadParamQuery,
  auth: Option<String>,
  stream: BodyStream,
) -> ApiResult<(PathFile, DateTime<Utc>)> {
  let auth = hash(auth)?;
  let secs = query
    .expire_time
    .unwrap_or(state.config.default_expire_time) as i64;
  let now = Utc::now();
  let expire_time = calc_expire_time(now, secs);
  let code_length = query
    .code_length
    .unwrap_or(state.config.default_code_length);
  let meta = MetaDataFile {
    create_at: now,
    expire_time,
    is_deleteable: query.deleteable.unwrap_or(true),
    max_download: query.max_download,
    auth,
    downloads: 0,
  };
  let path = loop {
    let path = generate_file_path(code_length, file_name);
    if state.db.exist(&path)? {
      continue;
    }
    match state.db.store(path.clone(), meta.clone()).await {
      Ok(_) => break path,
      Err(ApiError::ResourceExists(e)) => {
        debug!("key already exist: {e}");
        continue;
      }
      Err(e) => return Err(e),
    }
  };
  let file_path = state.config.fs.base_dir.join(&path);
  if let Err(e) = store_stream(&file_path, stream).await {
    state.db.delete(path).await?;
    return Err(e);
  }
  state.db.flush().await?;
  Ok((path, expire_time))
}

pub async fn info(
  state: &ApiState,
  code: &str,
  file_name: &str,
  auth: Option<String>,
) -> ApiResult<MetaDataFile> {
  let path = format!("{code}/{file_name}");
  let meta = state.db.fetch(&path)?.to_result()?;
  if let Some(max) = meta.max_download {
    if meta.downloads >= max {
      state.db.delete(path.clone()).await?;
      return Err(ApiError::NotFound(format!("{path} not found")));
    }
  }
  authenticate(auth, &meta.auth)?;
  Ok(meta)
}

pub async fn fetch(
  state: &ApiState,
  code: &str,
  file_name: &str,
  auth: Option<String>,
) -> ApiResult<ServeFile> {
  let path = format!("{code}/{file_name}");
  let meta = state.db.fetch_count(&path).await?.to_result()?;
  if let Some(max) = meta.max_download {
    if meta.downloads >= max {
      state.db.delete(path.clone()).await?;
      return Err(ApiError::NotFound(format!("{path} not found")));
    }
  }
  authenticate(auth, &meta.auth)?;
  read_file(&state.config.fs.base_dir.join(&path)).await
}

pub async fn delete(
  state: &ApiState,
  code: &str,
  file_name: &str,
  auth: Option<String>,
) -> ApiResult<()> {
  let path = format!("{code}/{file_name}");
  if let Some(meta) = state.db.fetch(&path)? {
    if meta.is_deleteable {
      authenticate(auth, &meta.auth)?;
      let file_path = state.config.fs.base_dir.join(&path);
      tokio::fs::remove_file(file_path).await?;
      state.db.delete(path).await?;
      state.db.flush().await?;
    } else {
      return Err(ApiError::PermissionDenied(format!(
        "{path} is not deleteable"
      )));
    }
  }
  Ok(())
}

pub async fn store_stream(file_path: &PathBuf, stream: BodyStream) -> ApiResult<()> {
  let stream = stream.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err));
  let stream = StreamReader::new(stream);
  futures_util::pin_mut!(stream);
  if let Some(p) = file_path.parent() {
    tokio::fs::create_dir_all(p).await?;
  }
  let mut file = BufWriter::new(File::create(file_path).await?);
  tokio::io::copy(&mut stream, &mut file).await?;
  Ok(())
}

pub async fn read_file(file_path: &PathBuf) -> ApiResult<ServeFile> {
  Ok(ServeFile::new(file_path))
}

pub fn authenticate(auth: Option<String>, hash: &Option<String>) -> ApiResult<()> {
  if let Some(hash) = hash {
    if !matches!(
      auth.map(|p| common::util::hash::argon_verify(p, hash)),
      Some(Ok(()))
    ) {
      return Err(ApiError::PermissionDenied(
        "user and password is invalid".to_string(),
      ));
    }
  }
  Ok(())
}

pub fn generate_file_path(code_length: usize, file_name: &str) -> String {
  let code = common::util::string::generate_random_string(code_length);
  format!("{code}/{file_name}")
}

pub fn hash(auth: Option<String>) -> ApiResult<Option<String>> {
  auth
    .as_ref()
    .map(common::util::hash::argon_hash)
    .transpose()
    .map_err(|e| ApiError::HashError(e.to_string()))
}

pub fn calc_expire_time(now: DateTime<Utc>, secs: i64) -> DateTime<Utc> {
  now + chrono::Duration::seconds(secs)
}
