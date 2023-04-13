use crate::error::{invalid_input_error, ApiResult};

pub fn validate(path: &str) -> ApiResult {
  let path = std::path::Path::new(path);
  let mut components = path.components().peekable();
  if let Some(first) = components.peek() {
    if !matches!(first, std::path::Component::Normal(_)) {
      return Err(invalid_input_error("file_name", "invalid"));
    }
  }
  if components.count() != 1 {
    return Err(invalid_input_error("file_name", "invalid"));
  }
  Ok(())
}
