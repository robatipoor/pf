use crate::{
  error::BodyResponseError,
  model::{
    request::UploadQueryParam,
    response::{MetaDataFileResponse, UploadResponse},
  },
  result::ApiResponseResult,
};

use log_derive::logfn;
use once_cell::sync::Lazy;
use reqwest::StatusCode;

pub static CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
  let disable_redirect = reqwest::redirect::Policy::custom(|attempt| attempt.stop());
  reqwest::Client::builder()
    .redirect(disable_redirect)
    .build()
    .unwrap()
});

pub struct PasteFileClient {
  pub client: &'static reqwest::Client,
  pub addr: String,
}

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
    query: &UploadQueryParam,
    file: Vec<u8>,
    auth: Option<(String, String)>,
  ) -> anyhow::Result<(StatusCode, ApiResponseResult<UploadResponse>)> {
    let file_part = reqwest::multipart::Part::bytes(file)
      .file_name(file_name)
      .mime_str(content_type)?;
    let form = reqwest::multipart::Form::new().part("file", file_part);
    let mut builder = self
      .client
      .post(format!("{}/upload", self.addr))
      .multipart(form)
      .query(query);
    if let Some((user, pass)) = auth {
      builder = builder.basic_auth(user, Some(pass));
    }
    let resp = builder.send().await?;
    Ok((resp.status(), resp.json().await?))
  }

  #[logfn(Info)]
  pub async fn download(
    &self,
    path_file: &str,
    auth: Option<(String, String)>,
  ) -> anyhow::Result<(StatusCode, ApiResponseResult<Vec<u8>>)> {
    let mut builder = self.client.get(format!("{}/{path_file}", self.addr));
    if let Some((user, pass)) = auth {
      builder = builder.basic_auth(user, Some(pass));
    }
    let resp = builder.send().await?;
    let status = resp.status();
    if !status.is_success() {
      let error = resp.json::<BodyResponseError>().await?;
      return Ok((status, ApiResponseResult::Err(error)));
    }
    Ok((status, ApiResponseResult::Ok(resp.bytes().await?.to_vec())))
  }

  #[logfn(Info)]
  pub async fn info(
    &self,
    path_file: &str,
    auth: Option<(String, String)>,
  ) -> anyhow::Result<(StatusCode, ApiResponseResult<MetaDataFileResponse>)> {
    let mut builder = self.client.get(format!("{}/info/{path_file}", self.addr));
    if let Some((user, pass)) = auth {
      builder = builder.basic_auth(user, Some(pass));
    }
    let resp = builder.send().await?;
    Ok((resp.status(), resp.json().await?))
  }

  #[logfn(Info)]
  pub async fn delete(
    &self,
    path_file: &str,
    auth: Option<(String, String)>,
  ) -> anyhow::Result<(StatusCode, ApiResponseResult)> {
    let mut builder = self.client.delete(format!("{}/{path_file}", self.addr));
    if let Some((user, pass)) = auth {
      builder = builder.basic_auth(user, Some(pass));
    }
    let resp = builder.send().await?;
    Ok((resp.status(), resp.json().await?))
  }
}
