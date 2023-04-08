use super::{PasteFileClient, CLIENT};
use once_cell::sync::Lazy;
use reqwest::StatusCode;

impl PasteFileClient {
  pub fn new(addr: &str) -> Self {
    Self {
      client: Lazy::force(&CLIENT),
      addr: addr.to_string(),
    }
  }

  pub async fn health_check(&self) -> anyhow::Result<StatusCode> {
    let resp = self
      .client
      .get(format!("{}/healthz", self.addr))
      .send()
      .await?;
    Ok(resp.status())
  }

  pub async fn upload_object(
    &self,
    filename: String,
    content_type: &str,
    file: Vec<u8>,
  ) -> anyhow::Result<StatusCode> {
    let file_part = reqwest::multipart::Part::bytes(file)
      .file_name(filename)
      .mime_str(content_type)?;
    let form = reqwest::multipart::Form::new().part("file", file_part);
    let resp = self
      .client
      .post(format!("{}/upload", self.addr))
      .multipart(form)
      .send()
      .await?;
    Ok(resp.status())
  }
}
