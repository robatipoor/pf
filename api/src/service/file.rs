use crate::configure::ApiConfig;
use crate::error::{ApiError, ApiResult, ToApiResult};
use crate::util::secret::{Secret, SecretHash};
use anyhow::anyhow;
use axum::extract::multipart::Field;
use axum::extract::Multipart;
use chrono::{DateTime, Utc};
use futures_util::TryStreamExt;
use sdk::dto::request::UploadQueryParam;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufWriter};
use tokio_util::bytes::BytesMut;
use tokio_util::io::StreamReader;
use tower_http::services::ServeFile;
use tracing::debug;

use crate::database::{FilePath, MetaDataFile};
use crate::server::ApiState;

const BYTE_TO_MEGABYTE: usize = 1024 * 1024;
const DEFAULT_BUF_SIZE: usize = 8192;

pub async fn store(
  state: &ApiState,
  query: &UploadQueryParam,
  secret: Option<Secret>,
  mut multipart: Multipart,
) -> ApiResult<(FilePath, DateTime<Utc>)> {
  let secret = secret.map(|s| s.hash()).transpose()?;
  let expire_secs = query
    .expire_secs
    .unwrap_or(state.config.default_expire_secs) as i64;
  let now = Utc::now();
  let expire_date_time = calc_expiration_date(now, expire_secs);
  let mut code_length = query
    .code_length
    .unwrap_or(state.config.default_code_length);
  let meta = MetaDataFile {
    created_at: now,
    expire_date_time,
    delete_manually: query.delete_manually.unwrap_or(true),
    max_download: query.max_download,
    secret,
    count_downloads: 0,
  };
  while let Some(field) = multipart.next_field().await? {
    let file_name = match field.file_name() {
      Some(file_name) => {
        crate::util::file_name::validate(file_name)?;
        file_name
      }
      None => continue,
    };
    let path = loop {
      let code = crate::util::string::generate_random_string(code_length);
      let path = FilePath {
        code,
        file_name: file_name.to_string(),
      };
      if !state.db.exist(&path)? {
        match state.db.store(path.clone(), meta.clone()).await {
          Ok(_) => break path,
          Err(ApiError::ResourceExistsError(e)) => {
            debug!("Key already exist: {e}");
            continue;
          }
          Err(e) => return Err(e),
        }
      }
      code_length += 1;
    };
    let file_path = path.fs_path(&state.config.fs.base_dir);
    if let Err(e) = store_stream(&file_path, field, state.config.max_upload_bytes_size).await {
      state.db.delete(path).await?;
      return Err(e);
    }
    state.db.flush().await?;
    return Ok((path, expire_date_time));
  }
  Err(ApiError::BadRequestError(
    "multipart/form-data empty body".to_string(),
  ))
}

pub async fn store_stream(file_path: &PathBuf, field: Field<'_>, max_size: usize) -> ApiResult<()> {
  let body_reader =
    StreamReader::new(field.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err)));
  futures_util::pin_mut!(body_reader);
  if let Some(parent) = file_path.parent() {
    tokio::fs::create_dir_all(parent).await?;
  }
  let mut file = BufWriter::new(File::create(file_path).await?);
  copy(file_path, &mut body_reader, &mut file, max_size).await?;
  Ok(())
}

pub async fn copy(
  file_path: &PathBuf,
  mut reader: impl AsyncRead + Unpin,
  mut writer: impl AsyncWrite + Unpin,
  max_size: usize,
) -> ApiResult<usize> {
  let mut buffer = BytesMut::with_capacity(DEFAULT_BUF_SIZE);
  let mut bytes_size = 0;
  loop {
    let bytes_read = reader.read_buf(&mut buffer).await?;
    if bytes_read == 0 {
      break;
    }
    bytes_size += bytes_read;
    if bytes_size > max_size {
      return handle_payload_too_large(file_path, writer, max_size).await;
    }
    writer.write_all(&buffer).await?;
    buffer.clear();
  }
  writer.flush().await?;
  Ok(bytes_size)
}

async fn handle_payload_too_large(
  file_path: &PathBuf,
  mut writer: impl AsyncWrite + Unpin,
  max_size: usize,
) -> ApiResult<usize> {
  writer.shutdown().await?;
  drop(writer);
  tokio::fs::remove_file(file_path).await?;
  Err(ApiError::PayloadTooLarge(format!(
    "The maximum allowed size for uploaded files is {}MB.",
    max_size / BYTE_TO_MEGABYTE
  )))
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
      return Err(ApiError::NotFoundError(format!(
        "{} not found",
        path.url_path()
      )));
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
  let file_path = FilePath {
    code: code.to_string(),
    file_name: file_name.to_string(),
  };
  let meta_data = state.db.fetch(&file_path)?.to_result()?;
  authorize_user(secret, &meta_data.secret)?;
  if let Some(max) = meta_data.max_download {
    if meta_data.count_downloads >= max {
      state.db.delete(file_path.clone()).await?;
      return Err(ApiError::NotFoundError(format!(
        "{} not found",
        file_path.url_path()
      )));
    } else if meta_data.count_downloads + 1 == max {
      // TODO set expire time
    }
  }
  let mut updated_meta_data = meta_data.clone();
  updated_meta_data.count_downloads += 1;
  state.db.update(&file_path, meta_data, updated_meta_data)?;
  Ok(read_file(&state.config, &file_path))
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
      return Err(ApiError::PermissionDeniedError(format!(
        "{} is not deletable",
        path.url_path()
      )));
    }
  }
  Ok(())
}

pub fn read_file(config: &ApiConfig, file_path: &FilePath) -> ServeFile {
  ServeFile::new(file_path.fs_path(&config.fs.base_dir))
}

pub fn authorize_user(secret: Option<Secret>, secret_hash: &Option<SecretHash>) -> ApiResult<()> {
  if let Some(hash) = secret_hash {
    return match secret.map(|s| s.verify(hash)) {
      Some(Ok(_)) => Ok(()),
      Some(Err(argon2::password_hash::Error::Password)) => Err(ApiError::PermissionDeniedError(
        "Secret token is invalid".to_string(),
      )),
      Some(Err(e)) => Err(ApiError::UnknownError(anyhow!(
        "An Unexpected error occurred: {e}",
      ))),
      None => Err(ApiError::PermissionDeniedError(
        "Authorization header required.".to_string(),
      )),
    };
  }
  Ok(())
}

pub fn calc_expiration_date(now: DateTime<Utc>, secs: i64) -> DateTime<Utc> {
  now + chrono::Duration::seconds(secs)
}
