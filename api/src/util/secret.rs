use crate::error::{ApiError, ApiResult};

#[derive(Debug)]
pub struct Secret {
  inner: String,
}

impl Secret {
  pub fn new(secret: String) -> Self {
    Self { inner: secret }
  }

  pub fn check(&self, hash: &str) -> Result<(), argon2::password_hash::Error> {
    crate::util::hash::argon_verify(&self.inner, hash)
  }

  pub fn hash(&self) -> ApiResult<String> {
    crate::util::hash::argon_hash(&self.inner).map_err(|e| ApiError::HashError(e.to_string()))
  }
}
