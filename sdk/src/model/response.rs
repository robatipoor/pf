use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageResponse {
  pub message: String,
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
  pub is_deletable: bool,
  pub max_download: Option<u32>,
  pub count_downloads: u32,
}
