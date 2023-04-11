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
  pub create_at: DateTime<Utc>,
  pub expire_time: DateTime<Utc>,
  pub is_deleteable: bool,
  pub max_download: Option<u32>,
  pub downloads: u32,
}
