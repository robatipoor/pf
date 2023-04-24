use axum::extract::BodyStream;
use chrono::{DateTime, Utc};
use common::error::{ApiError, ApiResult};
use common::model::request::UploadParamQuery;
use futures_util::TryStreamExt;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::BufWriter;
use tokio_util::io::StreamReader;
use tower_http::services::ServeFile;

use crate::database::{MetaDataFile, PathFile};
use crate::server::ApiState;

pub async fn store(
  state: &ApiState,
  file_name: &str,
  query: &UploadParamQuery,
  auth: Option<String>,
  stream: BodyStream,
) -> ApiResult<(PathFile, DateTime<Utc>)> {
  let auth = auth
    .as_ref()
    .map(common::util::hash::argon_hash)
    .transpose()
    .map_err(|e| ApiError::HashError(e.to_string()))?;
  let now = Utc::now();
  let expire_time = now + chrono::Duration::seconds(query.expire_time.unwrap_or(7200) as i64);
  let path = loop {
    let code: String = common::util::string::generate_random_string(query.length_code.unwrap_or(3));
    let path = format!("{code}/{file_name}");
    if !state.db.exist(&path).await {
      let meta = MetaDataFile {
        create_at: now,
        expire_time,
        is_deleteable: query.deleteable.unwrap_or(true),
        max_download: query.max_download,
        auth,
        downloads: 0,
      };
      state.db.store(path.clone(), meta).await?;
      break path;
    }
  };
  let file_path = state.config.fs.base_dir.join(&path);
  store_stream(&file_path, stream).await?;
  Ok((path, expire_time))
}

pub async fn info(
  state: &ApiState,
  code: &str,
  file_name: &str,
  auth: Option<String>,
) -> ApiResult<MetaDataFile> {
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
  if let Some(meta) = state.db.fetch(&path).await {
    if meta.is_deleteable {
      authenticate(auth, &meta.auth)?;
      let file_path = state.config.fs.base_dir.join(&path);
      state.db.delete(path).await;
      tokio::fs::remove_file(file_path).await?;
    } else {
      return Err(ApiError::PermissionDenied(format!(
        "it is not possible to delete {path}"
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
        "user and password invalid".to_string(),
      ));
    }
  }
  Ok(())
}
