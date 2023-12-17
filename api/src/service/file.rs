use crate::error::{ApiError, ApiResult, ToApiResult};
use axum::extract::Multipart;
use chrono::{DateTime, Utc};
use futures_util::TryStreamExt;
use sdk::model::request::UploadParamQuery;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::BufWriter;
use tokio_util::io::StreamReader;
use tower_http::services::ServeFile;
use tracing::debug;

use crate::database::{FilePath, MetaDataFile};
use crate::server::ApiState;

pub async fn store(
  state: &ApiState,
  file_name: &str,
  query: &UploadParamQuery,
  auth: Option<String>,
  multipart: Multipart,
) -> ApiResult<(FilePath, DateTime<Utc>)> {
  let auth = hash(auth)?;
  let secs = query
    .expire_time
    .unwrap_or(state.config.default_expire_time) as i64;
  let now = Utc::now();
  let expiration_date = calc_expiration_date(now, secs);
  let code_length = query
    .code_length
    .unwrap_or(state.config.default_code_length);
  let meta = MetaDataFile {
    created_at: now,
    expiration_date,
    is_deletable: query.deletable.unwrap_or(true),
    max_download: query.max_download,
    auth,
    count_downloads: 0,
  };
  let path = loop {
    let code = crate::util::string::generate_random_string(code_length);
    let path = FilePath {
      code,
      file_name: file_name.to_string(),
    };
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
  let file_path = path.fs_path(&state.config.fs.base_dir);
  if let Err(e) = store_stream(&file_path, multipart).await {
    state.db.delete(path).await?;
    return Err(e);
  }
  state.db.flush().await?;
  Ok((path, expiration_date))
}

pub async fn info(
  state: &ApiState,
  code: &str,
  file_name: &str,
  auth: Option<String>,
) -> ApiResult<MetaDataFile> {
  let path = FilePath {
    code: code.to_string(),
    file_name: file_name.to_string(),
  };
  let meta = state.db.fetch(&path)?.to_result()?;
  if let Some(max) = meta.max_download {
    if meta.count_downloads >= max {
      state.db.delete(path.clone()).await?;
      return Err(ApiError::NotFound(format!("{} not found", path.url_path())));
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
  let path = FilePath {
    code: code.to_string(),
    file_name: file_name.to_string(),
  };
  let meta = state.db.fetch_count(&path).await?.to_result()?;
  if let Some(max) = meta.max_download {
    if meta.count_downloads >= max {
      state.db.delete(path.clone()).await?;
      return Err(ApiError::NotFound(format!("{} not found", path.url_path())));
    }
  }
  authenticate(auth, &meta.auth)?;
  read_file(&state.config.fs.base_dir.join(&path.url_path())).await
}

pub async fn delete(
  state: &ApiState,
  code: &str,
  file_name: &str,
  auth: Option<String>,
) -> ApiResult<()> {
  let path = FilePath {
    code: code.to_string(),
    file_name: file_name.to_string(),
  };
  if let Some(meta) = state.db.fetch(&path)? {
    if meta.is_deletable {
      authenticate(auth, &meta.auth)?;
      let file_path = path.fs_path(&state.config.fs.base_dir);
      tokio::fs::remove_file(file_path).await?;
      state.db.delete(path).await?;
      state.db.flush().await?;
    } else {
      return Err(ApiError::PermissionDenied(format!(
        "{} is not deletable",
        path.url_path()
      )));
    }
  }
  Ok(())
}

pub async fn store_stream(file_path: &PathBuf, mut multipart: Multipart) -> ApiResult<()> {
  while let Ok(Some(field)) = multipart.next_field().await {
    let _file_name = if let Some(file_name) = field.file_name() {
      file_name.to_owned()
    } else {
      continue;
    };
    let body_with_io_error =
      field.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err));
    let body_reader = StreamReader::new(body_with_io_error);
    futures_util::pin_mut!(body_reader);
    if let Some(p) = file_path.parent() {
      tokio::fs::create_dir_all(p).await?;
    }
    let mut file = BufWriter::new(File::create(file_path).await?);
    tokio::io::copy(&mut body_reader, &mut file).await?;
  }

  Ok(())
}

pub async fn read_file(file_path: &PathBuf) -> ApiResult<ServeFile> {
  Ok(ServeFile::new(file_path))
}

pub fn authenticate(auth: Option<String>, hash: &Option<String>) -> ApiResult<()> {
  if let Some(hash) = hash {
    if !matches!(
      auth.map(|p| crate::util::hash::argon_verify(p, hash)),
      Some(Ok(()))
    ) {
      return Err(ApiError::PermissionDenied(
        "user and password is invalid".to_string(),
      ));
    }
  }
  Ok(())
}

pub fn hash(auth: Option<String>) -> ApiResult<Option<String>> {
  auth
    .as_ref()
    .map(crate::util::hash::argon_hash)
    .transpose()
    .map_err(|e| ApiError::HashError(e.to_string()))
}

pub fn calc_expiration_date(now: DateTime<Utc>, secs: i64) -> DateTime<Utc> {
  now + chrono::Duration::seconds(secs)
}
