pub fn create_url(
  base_url: &str,
  code: &str,
  file_name: &str,
) -> Result<url::Url, url::ParseError> {
  url::Url::parse(&format!("{base_url}/{}/{}", code, file_name))
}
