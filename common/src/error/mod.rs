use crate::model::response::MessageResponse;
use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use serde::{Deserialize, Serialize};

pub type ApiResult<T = ()> = std::result::Result<T, ApiError>;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
  #[error(transparent)]
  InvalidInput(#[from] validator::ValidationErrors),
  #[error("bad request {0}")]
  BadRequest(String),
  #[error("resource not found {0}")]
  NotFound(String),
  #[error("{0}")]
  PermissionDenied(String),
  #[error("resource not available {0}")]
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
      Unknown(err) => (
        "UNKNOWN_ERROR",
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(tag = "type", rename = "ServiceError")]
pub struct BodyResponseError {
  pub error_type: String,
  pub error: String,
}

impl BodyResponseError {
  pub fn new(error_type: &str, error_message: String) -> Self {
    Self {
      error_type: error_type.to_string(),
      error: error_message,
    }
  }
}

impl IntoResponse for ApiError {
  fn into_response(self) -> Response {
    let (body, status_code) = self.response();
    (status_code, Json(body)).into_response()
  }
}

pub fn invalid_input_error(feild: &'static str, message: &'static str) -> ApiError {
  let mut err = validator::ValidationErrors::new();
  err.add(
    feild,
    validator::ValidationError {
      code: std::borrow::Cow::from("1"),
      message: Some(std::borrow::Cow::Borrowed(message)),
      params: std::collections::HashMap::new(),
    },
  );
  ApiError::InvalidInput(err)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ApiResponseResult<T = MessageResponse> {
  Ok(T),
  Err(BodyResponseError),
}

impl<T> ApiResponseResult<T> {
  pub fn is_ok(&self) -> bool {
    matches!(*self, Self::Ok(_))
  }

  pub fn is_err(&self) -> bool {
    matches!(*self, Self::Err(_))
  }

  pub fn unwrap(self) -> T {
    match self {
      Self::Ok(t) => t,
      Self::Err(e) => panic!("called `AppResult::unwrap()` on an `Err` value {:?}", &e),
    }
  }
}
