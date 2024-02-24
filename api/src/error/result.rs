use super::ApiError;

pub type ApiResult<T = ()> = std::result::Result<T, ApiError>;

pub trait ToApiResult<T> {
  fn to_result(self, resource: &str) -> ApiResult<T>;
}

impl<T> ToApiResult<T> for Option<T> {
  fn to_result(self, resource: &str) -> ApiResult<T> {
    self.ok_or_else(|| ApiError::NotFoundError(format!("{resource} not found")))
  }
}
