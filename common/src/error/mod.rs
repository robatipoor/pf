use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use serde::{Deserialize, Serialize};
use std::{net::AddrParseError, time::SystemTimeError};
use strum_macros::Display;
use tokio::task::JoinError;

pub type ApiResult<T = ()> = std::result::Result<T, ApiError>;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
  #[error(transparent)]
  InvalidInput(#[from] validator::ValidationErrors),
  #[error("bad request {0}")]
  BadRequest(String),
  #[error("resource not found {0}")]
  NotFound(String),
  #[error("resource not available {0}")]
  NotAvailable(String),
  #[error("resource {0} exists already")]
  ResourceExists(ResourceType),
  #[error(transparent)]
  ConfigError(#[from] config::ConfigError),
  #[error(transparent)]
  AddrParseError(#[from] AddrParseError),
  #[error(transparent)]
  IoError(#[from] std::io::Error),
  #[error(transparent)]
  ParseJsonError(#[from] serde_json::Error),
  #[error(transparent)]
  ReqwestError(#[from] reqwest::Error),
  #[error(transparent)]
  SystemTimeError(#[from] SystemTimeError),
  #[error(transparent)]
  SpawnTaskError(#[from] JoinError),
  #[error(transparent)]
  HyperError(#[from] hyper::Error),
  #[error(transparent)]
  Unknown(#[from] anyhow::Error),
}

#[derive(Debug, Display)]
pub enum ResourceType {
  Volume,
  Bucket,
  Object,
  File,
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

#[derive(Debug, thiserror::Error)]
pub enum TaskError {
  #[error(transparent)]
  ConfigError(#[from] config::ConfigError),
  #[error(transparent)]
  AddrParseError(#[from] AddrParseError),
  #[error(transparent)]
  IoError(#[from] std::io::Error),
  #[error(transparent)]
  ParseJsonError(#[from] serde_json::Error),
  #[error(transparent)]
  ReqwestError(#[from] reqwest::Error),
  #[error(transparent)]
  SystemTimeError(#[from] SystemTimeError),
  #[error(transparent)]
  SpawnTaskError(#[from] JoinError),
  #[error(transparent)]
  HyperError(#[from] hyper::Error),
}
