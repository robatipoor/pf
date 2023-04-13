use hyper::HeaderMap;

use crate::error::{invalid_input_error, ApiResult};

pub fn parse_basic_auth(headers: &HeaderMap) -> ApiResult<Option<String>> {
  if let Some(value) = headers.get("Authorization") {
    let bytes = &value.as_bytes()["Basic ".len()..];
    if let Some(non_space_pos) = bytes.iter().position(|b| *b != b' ') {
      let bytes = &bytes[non_space_pos..];
      let value = String::from_utf8(bytes.to_vec())
        .map_err(|_e| invalid_input_error("Authorization", "invalid auth header"))?;
      Ok(Some(value))
    } else {
      Err(invalid_input_error("Authorization", "invalid auth header"))
    }
  } else {
    Ok(None)
  }
}
