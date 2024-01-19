use crate::error::{ApiError, ApiResult};

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct Secret(String);

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct SecretHash(String);

impl Secret {
  pub fn new(secret: String) -> Self {
    Self(secret)
  }

  pub fn verify(&self, hash: &SecretHash) -> Result<(), argon2::password_hash::Error> {
    crate::util::hash::argon_verify(&self.0, &hash.0)
  }

  pub fn hash(&self) -> ApiResult<SecretHash> {
    crate::util::hash::argon_hash(&self.0)
      .map_err(|e| ApiError::HashError(e.to_string()))
      .map(|hash| SecretHash(hash))
  }
}
