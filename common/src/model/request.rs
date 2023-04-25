use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, Default)]
pub struct UploadParamQuery {
  pub max_download: Option<u32>,
  #[validate(range(min = 4, max = 100))]
  pub code_length: Option<usize>,
  #[validate(range(min = 10, max = 1000000))]
  pub expire_time: Option<u64>,
  pub deleteable: Option<bool>,
}
