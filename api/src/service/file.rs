use crate::error::{ApiError, ApiResult, ToApiResult};
use crate::util::secret::{Secret, SecretHash};
use anyhow::anyhow;
use axum::extract::multipart::Field;
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
  query: &UploadParamQuery,
  secret: Option<Secret>,
  mut multipart: Multipart,
) -> ApiResult<(FilePath, DateTime<Utc>)> {
  let secret = secret.map(|s| s.hash()).transpose()?;
  let expire_secs = query
    .expire_time
    .unwrap_or(state.config.default_expire_secs) as i64;
  let now = Utc::now();
  let expiration_date = calc_expiration_date(now, expire_secs);
  let code_length = query
    .code_length
    .unwrap_or(state.config.default_code_length);
  let meta = MetaDataFile {
    created_at: now,
    expiration_date,
    delete_manually: query.delete_manually.unwrap_or(true),
    max_download: query.max_download,
    secret,
    count_downloads: 0,
  };
  while let Ok(Some(field)) = multipart.next_field().await {
    let file_name = if let Some(file_name) = field.file_name() {
      crate::util::file_name::validate(file_name)?;
      file_name.to_owned()
    } else {
      continue;
    };
    let path = loop {
      let code = crate::util::string::generate_random_string(code_length);
      let path = FilePath {
        code,
        file_name: file_name.clone(),
      };
      if state.db.exist(&path)? {
        continue;
      }
      match state.db.store(path.clone(), meta.clone()).await {
        Ok(_) => break path,
        Err(ApiError::ResourceExists(e)) => {
          debug!("Key already exist: {e}");
          continue;
        }
        Err(e) => return Err(e),
      }
    };
    let file_path = path.fs_path(&state.config.fs.base_dir);
    if let Err(e) = store_stream(&file_path, field).await {
      state.db.delete(path).await?;
      return Err(e);
    }
    state.db.flush().await?;
    return Ok((path, expiration_date));
  }
  Err(ApiError::BadRequest(
    "multipart/form-data empty body".to_string(),
  ))
}

pub async fn store_stream(file_path: &PathBuf, field: Field<'_>) -> ApiResult<()> {
  let body_with_io_error = field.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err));
  let body_reader = StreamReader::new(body_with_io_error);
  futures_util::pin_mut!(body_reader);
  if let Some(p) = file_path.parent() {
    tokio::fs::create_dir_all(p).await?;
  }
  let mut file = BufWriter::new(File::create(file_path).await?);
  tokio::io::copy(&mut body_reader, &mut file).await?;
  Ok(())
}

pub async fn info(
  state: &ApiState,
  code: &str,
  file_name: &str,
  secret: Option<Secret>,
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
  authorize_user(secret, &meta.secret)?;
  Ok(meta)
}

pub async fn fetch(
  state: &ApiState,
  code: &str,
  file_name: &str,
  secret: Option<Secret>,
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
  authorize_user(secret, &meta.secret)?;
  read_file(&state.config.fs.base_dir.join(&path.url_path())).await
}

pub async fn delete(
  state: &ApiState,
  code: &str,
  file_name: &str,
  secret: Option<Secret>,
) -> ApiResult<()> {
  let path = FilePath {
    code: code.to_string(),
    file_name: file_name.to_string(),
  };
  if let Some(meta) = state.db.fetch(&path)? {
    if meta.delete_manually {
      authorize_user(secret, &meta.secret)?;
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

pub async fn read_file(file_path: &PathBuf) -> ApiResult<ServeFile> {
  Ok(ServeFile::new(file_path))
}

pub fn authorize_user(secret: Option<Secret>, secret_hash: &Option<SecretHash>) -> ApiResult<()> {
  if let Some(hash) = secret_hash {
    match secret.map(|s| s.verify(hash)) {
      Some(Ok(_)) => return Ok(()),
      Some(Err(e)) if e == argon2::password_hash::Error::Password => Err(
        ApiError::PermissionDenied("Secret token is invalid".to_string()),
      ),
      Some(Err(e)) => Err(ApiError::Unknown(anyhow!(
        "An Unexpected error occurred: {e}",
      ))),
      None => Err(ApiError::PermissionDenied(
        "Authorization header required.".to_string(),
      )),
    }
  } else {
    Ok(())
  }
}

pub fn calc_expiration_date(now: DateTime<Utc>, secs: i64) -> DateTime<Utc> {
  now + chrono::Duration::seconds(secs)
}
