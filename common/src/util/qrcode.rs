use qrcodegen::QrCode;
use qrcodegen::QrCodeEcc;

use crate::error::ApiResult;

pub fn encode(text: &str) -> ApiResult<String> {
  // TODO fix error
  let qr = QrCode::encode_text(text, QrCodeEcc::Medium).unwrap();
  Ok(qr_to_str(&qr, 4))
}

fn qr_to_str(qr: &QrCode, border: i32) -> String {
  let mut s = String::new();
  for y in -border..qr.size() + border {
    for x in -border..qr.size() + border {
      let c: char = if qr.get_module(x, y) { '█' } else { ' ' };
      s.push_str(&format!("{0}{0}", c));
    }
    s.push('\n');
  }
  s.push('\n');
  s
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  pub fn test_encode() {
    encode("Hello").unwrap();
  }
}
