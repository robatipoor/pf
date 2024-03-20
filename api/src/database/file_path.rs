use crate::error::{result::ApiResult, ApiError};
use pf_sdk::dto::FileUrlPath;
use serde::{Deserialize, Serialize};
use sled::IVec;
use std::path::PathBuf;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, fake::Dummy)]
pub struct FilePath {
  pub code: String,
  pub file_name: String,
}

impl TryFrom<&IVec> for FilePath {
  type Error = ApiError;

  fn try_from(value: &IVec) -> ApiResult<Self> {
    let value = bincode::deserialize::<Self>(value)?;
    Ok(value)
  }
}

impl TryFrom<IVec> for FilePath {
  type Error = ApiError;

  fn try_from(value: IVec) -> ApiResult<Self> {
    Self::try_from(&value)
  }
}

impl TryFrom<&FilePath> for IVec {
  type Error = ApiError;
  fn try_from(value: &FilePath) -> ApiResult<IVec> {
    let value = bincode::serialize(value)?;
    Ok(IVec::from(value))
  }
}

impl TryFrom<FilePath> for IVec {
  type Error = ApiError;
  fn try_from(value: FilePath) -> ApiResult<IVec> {
    Self::try_from(&value)
  }
}

impl From<FilePath> for FileUrlPath {
  fn from(value: FilePath) -> Self {
    Self {
      code: value.code,
      file_name: value.file_name,
    }
  }
}

impl From<&FilePath> for PathBuf {
  fn from(value: &FilePath) -> Self {
    PathBuf::from(value.to_string())
  }
}

impl std::fmt::Display for FilePath {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}/{}", self.code, self.file_name)
  }
}
