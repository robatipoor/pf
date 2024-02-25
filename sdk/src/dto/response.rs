use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageResponse {
  pub message: String,
}

impl MessageResponse {
  pub fn ok() -> Self {
    Self {
      message: "Ok".to_string(),
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadResponse {
  pub expire_date_time: DateTime<Utc>,
  pub url: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub qr_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetaDataFileResponse {
  pub created_at: DateTime<Utc>,
  pub expire_date_time: DateTime<Utc>,
  pub allow_manual_deletion: bool,
  pub max_download: Option<u32>,
  pub count_downloads: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct BodyResponseError {
  pub error_type: String,
  pub error_message: String,
}

impl BodyResponseError {
  pub fn new(error_type: impl Into<String>, error_message: impl Into<String>) -> Self {
    Self {
      error_type: error_type.into(),
      error_message: error_message.into(),
    }
  }
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
      ApiResponseResult::Ok(t) => t,
      ApiResponseResult::Err(e) => {
        panic!("called `ApiResponseResult::unwrap()` on an `Err` value {e:?}")
      }
    }
  }

  pub fn map<U, F: FnOnce(T) -> U>(self, op: F) -> ApiResponseResult<U> {
    match self {
      ApiResponseResult::Ok(t) => ApiResponseResult::Ok(op(t)),
      ApiResponseResult::Err(e) => ApiResponseResult::Err(e),
    }
  }
}
