use serde::{Deserialize, Serialize};

use crate::{error::BodyResponseError, model::response::MessageResponse};

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
}
