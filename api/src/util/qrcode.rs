use base64::{engine, Engine};
use qrcode::QrCode;

use crate::error::ApiResult;

pub const BASE64_ENGIN: engine::GeneralPurpose =
  engine::GeneralPurpose::new(&base64::alphabet::STANDARD, engine::general_purpose::PAD);

pub fn encode(text: &str) -> ApiResult<String> {
  let qrcode = QrCode::new(text.as_bytes())?
    .render::<char>()
    .quiet_zone(false)
    .module_dimensions(1, 1)
    .build();
  Ok(BASE64_ENGIN.encode(qrcode))
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
