use base64::engine;

pub const BASE64_ENGIN: engine::GeneralPurpose =
  engine::GeneralPurpose::new(&base64::alphabet::STANDARD, engine::general_purpose::PAD);
