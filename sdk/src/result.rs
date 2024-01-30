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
