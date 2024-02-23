use garde::Validate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Validate, Default)]
pub struct UploadQueryParam {
  #[garde(range(min = 1, max = 100_000_000))]
  pub max_download: Option<u32>,
  #[garde(range(min = 3, max = 100))]
  pub code_length: Option<usize>,
  #[garde(range(min = 1, max = 100_000_000))]
  pub expire_secs: Option<u64>,
  #[garde(skip)]
  pub delete_manually: Option<bool>,
  #[garde(skip)]
  pub qr_code_format: Option<QrCodeFormat>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub enum QrCodeFormat {
  #[serde(rename = "text")]
  Text,
  #[serde(rename = "image")]
  Image,
}
