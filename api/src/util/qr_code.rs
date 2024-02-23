use std::io::Cursor;

use base64::{engine::general_purpose::STANDARD, Engine};
use image::{ImageOutputFormat, Luma};
use qrcode::QrCode;

use crate::error::result::ApiResult;

pub fn encode_to_text_format(text: &str) -> ApiResult<String> {
  let qr_code = QrCode::new(text.as_bytes())?
    .render::<char>()
    .quiet_zone(true)
    .module_dimensions(2, 1)
    .build();
  Ok(STANDARD.encode(qr_code))
}

pub fn encode_to_image_format(text: &str) -> ApiResult<String> {
  let qr_code: image::ImageBuffer<Luma<u8>, Vec<u8>> = QrCode::new(text.as_bytes())?
    .render::<Luma<u8>>()
    .quiet_zone(true)
    .module_dimensions(20, 20)
    .build();
  let mut buff = Cursor::new(Vec::new());
  qr_code.write_to(&mut buff, ImageOutputFormat::Png)?;
  Ok(STANDARD.encode(buff.into_inner()))
}

#[cfg(test)]
mod tests {
  use super::*;
  use fake::{Fake, Faker};

  #[test]
  pub fn test_encode_to_text_format() {
    let qr_code = encode_to_text_format(&Faker.fake::<String>()).unwrap();
    let _content = STANDARD.decode(qr_code.as_bytes()).unwrap();
  }

  #[test]
  pub fn test_encode_to_image_format() {
    let qr_code = encode_to_image_format(&Faker.fake::<String>()).unwrap();
    let _content = STANDARD.decode(qr_code.as_bytes()).unwrap();
  }
}
