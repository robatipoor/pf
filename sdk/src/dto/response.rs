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
  pub expire_time: DateTime<Utc>,
  pub url: String,
  pub qrcode: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetaDataFileResponse {
  pub created_at: DateTime<Utc>,
  pub expiration_date: DateTime<Utc>,
  pub delete_manually: bool,
  pub max_download: Option<u32>,
  pub count_downloads: u32,
}
