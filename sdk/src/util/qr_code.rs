use std::io::Cursor;

use base64::{engine::general_purpose::STANDARD, Engine};
use image::{ImageOutputFormat, Luma};
use qrcode::QrCode;

pub fn generate_text_qr_code(input: &str) -> anyhow::Result<String> {
  Ok(
    QrCode::new(input.as_bytes())?
      .render::<char>()
      .quiet_zone(true)
      .module_dimensions(2, 1)
      .build(),
  )
}

pub fn generate_base64_text_qr_code(input: &str) -> anyhow::Result<String> {
  Ok(STANDARD.encode(generate_text_qr_code(input)?))
}

pub fn generate_base64_png_qr_code(input: &str) -> anyhow::Result<String> {
  let qr_code: image::ImageBuffer<Luma<u8>, Vec<u8>> = QrCode::new(input.as_bytes())?
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
  pub fn test_generate_base64_text_qr_code() {
    let qr_code = generate_base64_text_qr_code(&Faker.fake::<String>()).unwrap();
    let _content = STANDARD.decode(qr_code.as_bytes()).unwrap();
  }

  #[test]
  pub fn test_generate_base64_png_qr_code() {
    let qr_code = generate_base64_png_qr_code(&Faker.fake::<String>()).unwrap();
    let _content = STANDARD.decode(qr_code.as_bytes()).unwrap();
  }
}
