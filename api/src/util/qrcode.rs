use base64::{engine::general_purpose::STANDARD, Engine};
use qrcode::QrCode;

use crate::error::ApiResult;

pub fn encode(text: &str) -> ApiResult<String> {
  let qrcode = QrCode::new(text.as_bytes())?
    .render::<char>()
    .quiet_zone(false)
    .module_dimensions(2, 1)
    .build();
  Ok(STANDARD.encode(qrcode))
}

#[cfg(test)]
mod tests {
  use super::*;
  use fake::{Fake, Faker};

  #[test]
  pub fn test_encode_qrcode() {
    let qr_code = encode(&Faker.fake::<String>()).unwrap();
    println!("{qr_code}")
  }
}
