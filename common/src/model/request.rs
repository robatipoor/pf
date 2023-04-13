use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UploadParamQuery {
  pub max_download: Option<u32>,
  #[validate(range(min = 4, max = 100))]
  pub length_code: Option<usize>,
  #[validate(range(min = 10, max = 1000000))]
  pub expire_time: Option<u64>,
  pub deleteable: Option<bool>,
}

impl Default for UploadParamQuery {
  fn default() -> Self {
    Self {
      max_download: None,
      length_code: Some(4),
      expire_time: Some(7200),
      deleteable: Some(true),
    }
  }
}
