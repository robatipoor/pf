use std::{
  ops::Deref,
  path::{Path, PathBuf},
};

use crate::dto::{
  request::UploadQueryParam,
  response::{ApiResponseResult, BodyResponseError, MetaDataFileResponse, UploadResponse},
  FileUrlPath,
};

use futures_util::StreamExt;
use log_derive::logfn;
use once_cell::sync::Lazy;
use reqwest::StatusCode;
use tokio::io::{AsyncRead, AsyncWriteExt};
use tokio_util::io::ReaderStream;

pub static CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
  let disable_redirect = reqwest::redirect::Policy::custom(|attempt| attempt.stop());
  reqwest::Client::builder()
    .redirect(disable_redirect)
    .build()
    .unwrap()
});

pub struct PasteFileClient {
  pub inner: reqwest::Client,
  pub addr: String,
}

impl PasteFileClient {
  pub fn new(addr: String) -> Self {
    Self {
      inner: CLIENT.clone(),
      addr,
    }
  }

  #[logfn(Info)]
  pub async fn health_check(&self) -> anyhow::Result<(StatusCode, ApiResponseResult)> {
    let resp = self.get(format!("{}/healthz", self.addr)).send().await?;
    Ok((resp.status(), resp.json().await?))
  }

  #[logfn(Info)]
  pub async fn upload_file_part(
    &self,
    file_part: reqwest::multipart::Part,
    query: &UploadQueryParam,
    auth: Option<(String, String)>,
  ) -> anyhow::Result<(StatusCode, ApiResponseResult<UploadResponse>)> {
    let form = reqwest::multipart::Form::new().part("file", file_part);
    let mut builder = self
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
  pub async fn upload(
    &self,
    file_name: String,
    content_type: &str,
    file: Vec<u8>,
    query: &UploadQueryParam,
    auth: Option<(String, String)>,
  ) -> anyhow::Result<(StatusCode, ApiResponseResult<UploadResponse>)> {
    let file_part = reqwest::multipart::Part::bytes(file)
      .file_name(file_name)
      .mime_str(content_type)?;
    self.upload_file_part(file_part, query, auth).await
  }

  pub async fn upload_reader<R>(
    &self,
    file_name: String,
    content_type: &str,
    reader: R,
    query: &UploadQueryParam,
    auth: Option<(String, String)>,
  ) -> anyhow::Result<(StatusCode, ApiResponseResult<UploadResponse>)>
  where
    R: AsyncRead + Send + Unpin + 'static,
  {
    let file_part =
      reqwest::multipart::Part::stream(reqwest::Body::wrap_stream(ReaderStream::new(reader)))
        .file_name(file_name.clone())
        .mime_str(content_type)?;
    self.upload_file_part(file_part, query, auth).await
  }

  #[logfn(Info)]
  pub async fn upload_file(
    &self,
    source: &Path,
    query: &UploadQueryParam,
    auth: Option<(String, String)>,
  ) -> anyhow::Result<(StatusCode, ApiResponseResult<UploadResponse>)> {
    let file_name = crate::util::file::get_file_name(source)?;
    let content_type = crate::util::file::get_content_type(source)?;
    let file = tokio::fs::File::open(source).await?;
    self
      .upload_reader(file_name, &content_type, file, query, auth)
      .await
  }

  #[logfn(Info)]
  pub async fn download(
    &self,
    url_path: &FileUrlPath,
    auth: Option<(String, String)>,
  ) -> anyhow::Result<(StatusCode, ApiResponseResult<Vec<u8>>)> {
    let mut builder = self.get(url_path.to_url(&self.addr)?);
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
  pub async fn download_file(
    &self,
    url_path: &FileUrlPath,
    auth: Option<(String, String)>,
    mut destination: PathBuf,
  ) -> anyhow::Result<(StatusCode, ApiResponseResult<PathBuf>)> {
    if destination.is_dir() {
      destination.push(&url_path.file_name);
    }
    let mut builder = self.get(url_path.to_url(&self.addr)?);
    if let Some((user, pass)) = auth {
      builder = builder.basic_auth(user, Some(pass));
    }
    let resp = builder.send().await?;
    let status = resp.status();
    if !status.is_success() {
      let error = resp.json::<BodyResponseError>().await?;
      return Ok((status, ApiResponseResult::Err(error)));
    }
    if let Some(parent) = destination.parent() {
      tokio::fs::create_dir_all(parent).await?;
    }
    let mut file = tokio::fs::File::create(&destination).await?;
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
      let chunk = chunk?;
      file.write_all(&chunk).await?;
    }
    Ok((status, ApiResponseResult::Ok(destination)))
  }

  #[logfn(Info)]
  pub async fn info(
    &self,
    url_path: &FileUrlPath,
    auth: Option<(String, String)>,
  ) -> anyhow::Result<(StatusCode, ApiResponseResult<MetaDataFileResponse>)> {
    let mut builder = self.get(format!("{}/info/{}", self.addr, url_path));
    if let Some((user, pass)) = auth {
      builder = builder.basic_auth(user, Some(pass));
    }
    let resp = builder.send().await?;
    Ok((resp.status(), resp.json().await?))
  }

  #[logfn(Info)]
  pub async fn delete(
    &self,
    url_path: &FileUrlPath,
    auth: Option<(String, String)>,
  ) -> anyhow::Result<(StatusCode, ApiResponseResult)> {
    let mut builder = self.inner.delete(url_path.to_url(&self.addr)?);
    if let Some((user, pass)) = auth {
      builder = builder.basic_auth(user, Some(pass));
    }
    let resp = builder.send().await?;
    Ok((resp.status(), resp.json().await?))
  }
}

impl Deref for PasteFileClient {
  type Target = reqwest::Client;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}
