use crate::{error::ApiResponseResult, model::response::UploadResponse};

use super::{PasteFileClient, CLIENT};
use log_derive::logfn;
use once_cell::sync::Lazy;
use reqwest::StatusCode;

impl PasteFileClient {
  pub fn new(addr: &str) -> Self {
    Self {
      client: Lazy::force(&CLIENT),
      addr: addr.to_string(),
    }
  }

  #[logfn(Info)]
  pub async fn health_check(&self) -> anyhow::Result<(StatusCode, ApiResponseResult)> {
    let resp = self
      .client
      .get(format!("{}/healthz", self.addr))
      .send()
      .await?;
    Ok((resp.status(), resp.json().await?))
  }

  pub async fn home_page(&self) -> anyhow::Result<(StatusCode, String)> {
    let resp = self.client.get(format!("{}/web", self.addr)).send().await?;
    Ok((resp.status(), resp.text().await?))
  }

  #[logfn(Info)]
  pub async fn upload(
    &self,
    file_name: String,
    content_type: &str,
    file: Vec<u8>,
  ) -> anyhow::Result<(StatusCode, ApiResponseResult<UploadResponse>)> {
    let file_part = reqwest::multipart::Part::bytes(file)
      .file_name(file_name.clone())
      .mime_str(content_type)?;
    let form = reqwest::multipart::Form::new().part("file", file_part);
    let resp = self
      .client
      .post(format!("{}/upload/{file_name}", self.addr))
      .multipart(form)
      .send()
      .await?;
    Ok((resp.status(), resp.json().await?))
  }

  #[logfn(Info)]
  pub async fn download(
    &self,
    path_file: String,
    auth: Option<(String, String)>,
  ) -> anyhow::Result<StatusCode> {
    let mut builder = self.client.delete(format!("{}/{path_file}", self.addr));
    if let Some((user, pass)) = auth {
      builder = builder.basic_auth(user, Some(pass));
    }
    Ok(builder.send().await?.status())
  }

  #[logfn(Info)]
  pub async fn info(
    &self,
    path_file: String,
    auth: Option<(String, String)>,
  ) -> anyhow::Result<StatusCode> {
    let mut builder = self.client.delete(format!("{}/{path_file}", self.addr));
    if let Some((user, pass)) = auth {
      builder = builder.basic_auth(user, Some(pass));
    }
    Ok(builder.send().await?.status())
  }

  #[logfn(Info)]
  pub async fn delete(
    &self,
    path_file: String,
    auth: Option<(String, String)>,
  ) -> anyhow::Result<StatusCode> {
    let mut builder = self.client.delete(format!("{}/{path_file}", self.addr));
    if let Some((user, pass)) = auth {
      builder = builder.basic_auth(user, Some(pass));
    }
    Ok(builder.send().await?.status())
  }
}
