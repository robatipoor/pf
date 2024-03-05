use sdk::dto::request::QrCodeFormat;

use crate::error::result::ApiResult;

pub fn generate_qr_code(qr_code_format: QrCodeFormat, input: &str) -> ApiResult<String> {
  match qr_code_format {
    QrCodeFormat::Text => Ok(sdk::util::qr_code::generate_base64_text_qr_code(input)?),
    QrCodeFormat::Image => Ok(sdk::util::qr_code::generate_base64_png_qr_code(input)?),
  }
}
