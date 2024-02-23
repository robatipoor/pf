use base64::{engine::general_purpose::STANDARD, Engine};
use image::Luma;
use qrcode::QrCode;

use crate::error::ApiResult;

pub fn encode_to_text_format(text: &str) -> ApiResult<String> {
  let qr_code = QrCode::new(text.as_bytes())?
    .render::<char>()
    .quiet_zone(false)
    .module_dimensions(2, 1)
    .build();
  Ok(STANDARD.encode(qr_code))
}

pub fn encode_to_image_format(text: &str) -> ApiResult<String> {
  let qr_code = QrCode::new(text.as_bytes())?
    .render::<Luma<u8>>()
    .quiet_zone(false)
    .module_dimensions(2, 1)
    .build();
  Ok(STANDARD.encode(qr_code.into_vec()))
}

#[cfg(test)]
mod tests {
  use super::*;
  use fake::{Fake, Faker};

  #[test]
  pub fn test_encode_to_text_format() {
    let _qr_code = encode_to_text_format(&Faker.fake::<String>()).unwrap();
  }
}
