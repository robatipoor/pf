use base64::Engine;
use qrcode::QrCode;
use sdk::util::base64::BASE64_ENGIN;

use crate::error::ApiResult;

pub fn encode(text: &str) -> ApiResult<String> {
  let qrcode = QrCode::new(text.as_bytes())?
    .render::<char>()
    .quiet_zone(false)
    .module_dimensions(2, 1)
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
