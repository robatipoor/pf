use crate::{
  error::{result::ApiResult, ApiError},
  util::secret::SecretHash,
};
use chrono::{DateTime, Utc};
use pf_sdk::dto::response::MetaDataFileResponse;
use serde::{Deserialize, Serialize};
use sled::IVec;

#[derive(Debug, Clone, Serialize, Deserialize, fake::Dummy)]
pub struct MetaDataFile {
  pub created_at: DateTime<Utc>,
  pub expire_date_time: DateTime<Utc>,
  pub secret: Option<SecretHash>,
  pub manual_deletion: bool,
  pub max_download: Option<u32>,
  pub count_downloads: u32,
}

impl TryFrom<&[u8]> for MetaDataFile {
  type Error = ApiError;

  fn try_from(value: &[u8]) -> ApiResult<Self> {
    let value = bincode::deserialize::<Self>(value)?;
    Ok(value)
  }
}

impl TryFrom<&IVec> for MetaDataFile {
  type Error = ApiError;

  fn try_from(value: &IVec) -> ApiResult<Self> {
    let value = bincode::deserialize::<Self>(value)?;
    Ok(value)
  }
}

impl TryFrom<IVec> for MetaDataFile {
  type Error = ApiError;

  fn try_from(value: IVec) -> ApiResult<Self> {
    Self::try_from(&value)
  }
}

impl TryFrom<&MetaDataFile> for IVec {
  type Error = ApiError;
  fn try_from(value: &MetaDataFile) -> ApiResult<IVec> {
    let value = bincode::serialize(value)?;
    Ok(IVec::from(value))
  }
}

impl TryFrom<MetaDataFile> for IVec {
  type Error = ApiError;
  fn try_from(value: MetaDataFile) -> ApiResult<IVec> {
    Self::try_from(&value)
  }
}

impl From<&MetaDataFile> for MetaDataFileResponse {
  fn from(value: &MetaDataFile) -> Self {
    MetaDataFileResponse {
      created_at: value.created_at,
      expire_date_time: value.expire_date_time,
      allow_manual_deletion: value.manual_deletion,
      max_download: value.max_download,
      count_downloads: value.count_downloads,
    }
  }
}
