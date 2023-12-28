use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use sdk::error::BodyResponseError;

pub type ApiResult<T = ()> = std::result::Result<T, ApiError>;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
  #[error(transparent)]
  InvalidInput(#[from] validator::ValidationErrors),
  #[error("bad request: {0}")]
  BadRequest(String),
  #[error("resource not found: {0}")]
  NotFound(String),
  #[error("{0}")]
  PermissionDenied(String),
  #[error("resource not available: {0}")]
  NotAvailable(String),
  #[error("resource {0} exists already")]
  ResourceExists(String),
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
  #[error("lock error: {0}")]
  LockError(String),
  #[error("duration out of range error: {0}")]
  DurationOutOfRangeError(#[from] chrono::OutOfRangeError),
  #[error(transparent)]
  Unknown(#[from] anyhow::Error),
}

impl ApiError {
  pub fn response(&self) -> (BodyResponseError, StatusCode) {
    use ApiError::*;
    let (error_type, error_message, status_code) = match self {
      InvalidInput(err) => (
        "INVALID_INPUT",
        err.to_string(),
        StatusCode::UNPROCESSABLE_ENTITY,
      ),
      BadRequest(err) => ("BAD_REQUEST", err.to_string(), StatusCode::BAD_REQUEST),
      PermissionDenied(err) => ("PERMISSION_DENIED", err.to_string(), StatusCode::FORBIDDEN),
      NotAvailable(err) => ("NOT_AVAILABLE", err.to_string(), StatusCode::NOT_FOUND),
      NotFound(err) => ("NOT_FOUND", err.to_string(), StatusCode::NOT_FOUND),
      ResourceExists(err) => ("RESOURCE_EXISTS", err.to_string(), StatusCode::CONFLICT),
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
      Unknown(err) => (
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

impl IntoResponse for ApiError {
  fn into_response(self) -> Response {
    let (body, status_code) = self.response();
    (status_code, Json(body)).into_response()
  }
}

pub fn invalid_input_error(field: &'static str, message: &'static str) -> ApiError {
  let mut err = validator::ValidationErrors::new();
  err.add(
    field,
    validator::ValidationError {
      code: std::borrow::Cow::from("1"),
      message: Some(std::borrow::Cow::Borrowed(message)),
      params: std::collections::HashMap::new(),
    },
  );
  ApiError::InvalidInput(err)
}

pub trait ToApiResult<T> {
  fn to_result(self) -> ApiResult<T>;
}

impl<T> ToApiResult<T> for Option<T> {
  fn to_result(self) -> ApiResult<T> {
    self.ok_or_else(|| ApiError::NotFound(format!("{} not found", std::any::type_name::<T>())))
  }
}
