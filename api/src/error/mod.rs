use axum::http::StatusCode;
use sdk::dto::response::BodyResponseError;

pub mod response;
pub mod result;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
  #[error(transparent)]
  InvalidInputError(#[from] garde::Report),
  #[error("bad request: {0}")]
  BadRequestError(String),
  #[error("payload too large: {0}")]
  PayloadTooLarge(String),
  #[error("resource not found: {0}")]
  NotFoundError(String),
  #[error("{0}")]
  PermissionDeniedError(String),
  #[error("resource not available: {0}")]
  NotAvailableError(String),
  #[error("resource {0} exists already")]
  ResourceExistsError(String),
  #[error(transparent)]
  ConfigError(#[from] config::ConfigError),
  #[error(transparent)]
  AddrParseError(#[from] std::net::AddrParseError),
  #[error(transparent)]
  IoError(#[from] std::io::Error),
  #[error(transparent)]
  ParseJsonError(#[from] serde_json::Error),
  #[error(transparent)]
  BincodeError(#[from] bincode::Error),
  #[error(transparent)]
  ReqwestError(#[from] reqwest::Error),
  #[error(transparent)]
  SystemTimeError(#[from] std::time::SystemTimeError),
  #[error("hash error {0}")]
  HashError(String),
  #[error(transparent)]
  SpawnTaskError(#[from] tokio::task::JoinError),
  #[error(transparent)]
  HyperError(#[from] hyper::Error),
  #[error(transparent)]
  DatabaseError(#[from] sled::Error),
  #[error(transparent)]
  Utf8Error(#[from] std::str::Utf8Error),
  #[error(transparent)]
  QrCodeError(#[from] qrcode::types::QrError),
  #[error(transparent)]
  MultipartError(#[from] axum::extract::multipart::MultipartError),
  #[error(transparent)]
  UrlError(#[from] url::ParseError),
  #[error(transparent)]
  ImageError(#[from] image::ImageError),
  #[error("lock error: {0}")]
  LockError(String),
  #[error("duration out of range error: {0}")]
  DurationOutOfRangeError(#[from] chrono::OutOfRangeError),
  #[error(transparent)]
  UnknownError(#[from] anyhow::Error),
}

impl ApiError {
  pub fn response(&self) -> (BodyResponseError, StatusCode) {
    use ApiError::*;
    let (error_type, error_message, status_code) = match self {
      InvalidInputError(err) => (
        "INVALID_INPUT",
        err.to_string(),
        StatusCode::UNPROCESSABLE_ENTITY,
      ),
      BadRequestError(err) => ("BAD_REQUEST", err.to_string(), StatusCode::BAD_REQUEST),
      PermissionDeniedError(err) => ("PERMISSION_DENIED", err.to_string(), StatusCode::FORBIDDEN),
      NotAvailableError(err) => ("NOT_AVAILABLE", err.to_string(), StatusCode::NOT_FOUND),
      NotFoundError(err) => ("NOT_FOUND", err.to_string(), StatusCode::NOT_FOUND),
      ResourceExistsError(err) => ("RESOURCE_EXISTS", err.to_string(), StatusCode::CONFLICT),
      ConfigError(err) => (
        "CONFIG_ERROR",
        err.to_string(),
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      AddrParseError(err) => (
        "ADDR_PARSE_ERROR",
        err.to_string(),
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      IoError(err) => (
        "IO_ERROR",
        err.to_string(),
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      ParseJsonError(err) => (
        "PARSE_JSON_ERROR",
        err.to_string(),
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      BincodeError(err) => (
        "PARSE_JSON_ERROR",
        err.to_string(),
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      ReqwestError(err) => (
        "REQWEST_ERROR",
        err.to_string(),
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      SystemTimeError(err) => (
        "SYSTEM_TIME_ERROR",
        err.to_string(),
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      HashError(err) => (
        "HASH_ERROR",
        err.to_string(),
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      SpawnTaskError(err) => (
        "SPAWN_TASK_ERROR",
        err.to_string(),
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      HyperError(err) => (
        "HYPER_ERROR",
        err.to_string(),
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      DatabaseError(err) => (
        "DATABASE_ERROR",
        err.to_string(),
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      Utf8Error(err) => (
        "UTF8_ERROR",
        err.to_string(),
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      QrCodeError(err) => (
        "QRCODE_ERROR",
        err.to_string(),
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      MultipartError(err) => (
        "MULTIPART_ERROR",
        err.to_string(),
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      PayloadTooLarge(err) => (
        "PAYLOAD_TOO_LARGE_ERROR",
        err.to_string(),
        StatusCode::PAYLOAD_TOO_LARGE,
      ),
      UrlError(err) => (
        "URL_ERROR",
        err.to_string(),
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      ImageError(err) => (
        "IMAGE_ERROR",
        err.to_string(),
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      UnknownError(err) => (
        "UNKNOWN_ERROR",
        err.to_string(),
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      LockError(err) => (
        "LOCK_ERROR",
        err.to_string(),
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      DurationOutOfRangeError(err) => (
        "DURATION_OUT_OF_RANGE",
        err.to_string(),
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
    };
    (
      BodyResponseError::new(error_type, error_message),
      status_code,
    )
  }
}

pub fn invalid_input_error(field: &'static str, message: &'static str) -> ApiError {
  let mut report = garde::Report::new();
  report.append(garde::Path::new(field), garde::Error::new(message));
  ApiError::InvalidInputError(report)
}
