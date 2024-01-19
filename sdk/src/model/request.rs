use garde::Validate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Validate, Default)]
pub struct UploadQueryParam {
  #[garde(range(min = 1))]
  pub max_download: Option<u32>,
  #[garde(range(min = 4, max = 100))]
  pub code_length: Option<usize>,
  #[garde(range(min = 1, max = 1000000))]
  pub expire_time: Option<u64>,
  #[garde(skip)]
  pub delete_manually: Option<bool>,
}
