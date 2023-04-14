use axum::extract::BodyStream;
use axum_extra::body::AsyncReadBody;
use chrono::{DateTime, Utc};
use common::error::{ApiError, ApiResult};
use common::model::request::UploadParamQuery;
use futures_util::TryStreamExt;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::BufWriter;
use tokio_util::io::StreamReader;

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
  let expire_time = now + chrono::Duration::seconds(query.expire_time.unwrap() as i64);
  let path = loop {
    let code: String = common::util::string::generate_random_string(query.length_code.unwrap());
    let path = format!("{code}/{file_name}");
    if !state.db.exist(&path).await {
      let meta = MetaDataFile {
        create_at: now,
        expire_time,
        is_deleteable: query.deleteable.unwrap(),
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
  auth: Option<String>,
) -> ApiResult<AsyncReadBody<File>> {
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
  if let Some(hash) = meta.auth.as_ref() {
    if !matches!(
      auth.map(|p| common::util::hash::argon_verify(p, hash)),
      Some(Ok(()))
    ) {
      return Err(ApiError::PermissionDenied("password invalid".to_string()));
    }
  }
  read_file(&state.config.fs.base_dir.join(&path)).await
}

pub async fn delete(state: &ApiState, code: &str, file_name: &str) -> ApiResult<()> {
  let path = format!("{code}/{file_name}");
  if let Some(info) = state.db.fetch(&path).await {
    if info.is_deleteable {
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
  let mut file = BufWriter::new(File::create(file_path).await?);
  tokio::io::copy(&mut stream, &mut file).await?;
  Ok(())
}

pub async fn read_file(file_path: &PathBuf) -> ApiResult<AsyncReadBody<File>> {
  let file = File::open(file_path).await?;
  Ok(AsyncReadBody::new(file))
}
