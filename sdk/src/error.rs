use serde::{Deserialize, Serialize};

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
