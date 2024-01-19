use hyper::HeaderMap;

use crate::error::{invalid_input_error, ApiResult};

use super::secret::Secret;

pub fn parse_basic_auth(headers: &HeaderMap) -> ApiResult<Option<Secret>> {
  if let Some(value) = headers.get("Authorization") {
    let bytes = &value.as_bytes()["Basic ".len()..];
    if let Some(non_space_pos) = bytes.iter().position(|b| *b != b' ') {
      let bytes = &bytes[non_space_pos..];
      let value = String::from_utf8(bytes.to_vec())
        .map_err(|_e| invalid_input_error("Authorization", "invalid auth header"))?;
      Ok(Some(Secret::new(value)))
    } else {
      Err(invalid_input_error("Authorization", "Invalid auth header"))
    }
  } else {
    Ok(None)
  }
}
