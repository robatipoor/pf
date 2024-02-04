use std::path::{Path, PathBuf};

use crate::{
  dto::{
    request::UploadQueryParam,
    response::{ApiResponseResult, BodyResponseError, MetaDataFileResponse, UploadResponse},
  },
  util::progress::progress_bar,
};

use futures_util::StreamExt;
use log_derive::logfn;
use once_cell::sync::Lazy;
use reqwest::StatusCode;
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;

pub static CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
  let disable_redirect = reqwest::redirect::Policy::custom(|attempt| attempt.stop());
  reqwest::Client::builder()
    .redirect(disable_redirect)
    .build()
    .unwrap()
});

pub struct PasteFileClient {
  pub client: reqwest::Client,
  pub addr: String,
}

impl PasteFileClient {
  pub fn new(addr: String) -> Self {
    Self {
      client: CLIENT.clone(),
      addr,
    }
  }

  pub fn new_client(addr: String, client: reqwest::Client) -> Self {
    Self { client, addr }
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

  #[logfn(Info)]
  async fn upload_file(
    &self,
    file_part: reqwest::multipart::Part,
    query: &UploadQueryParam,
    auth: Option<(String, String)>,
  ) -> anyhow::Result<(StatusCode, ApiResponseResult<UploadResponse>)> {
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
    self.upload_file(file_part, query, auth).await
  }

  #[logfn(Info)]
  pub async fn upload_from(
    &self,
    source: &Path,
    query: &UploadQueryParam,
    auth: Option<(String, String)>,
  ) -> anyhow::Result<(StatusCode, ApiResponseResult<UploadResponse>)> {
    let file_name = crate::util::file::get_file_name(source)?;
    let content_type = crate::util::file::get_content_type(source)?;
    let file = tokio::fs::File::open(source).await?;
    let file_part =
      reqwest::multipart::Part::stream(reqwest::Body::wrap_stream(ReaderStream::new(file)))
        .file_name(file_name)
        .mime_str(&content_type)?;
    self.upload_file(file_part, query, auth).await
  }

  #[logfn(Info)]
  pub async fn upload_with_progress_bar(
    &self,
    source: &Path,
    query: &UploadQueryParam,
    auth: Option<(String, String)>,
  ) -> anyhow::Result<(StatusCode, ApiResponseResult<UploadResponse>)> {
    let file_name = crate::util::file::get_file_name(source)?;
    let content_type = crate::util::file::get_content_type(source)?;
    let file = tokio::fs::File::open(source).await?;
    let total_size = file.metadata().await?.len();
    let mut reader_stream = ReaderStream::new(file);
    let pb = progress_bar(total_size)?;
    let mut uploaded = 0;
    let async_stream = async_stream::stream! {
        while let Some(chunk) = reader_stream.next().await {
            if let Ok(chunk) = &chunk {
                uploaded += chunk.len() as u64;
                pb.set_position(uploaded.min(total_size));
                if uploaded >= total_size {
                    pb.finish_with_message("File upload completed successfully.");
                }
            }
            yield chunk;
        }
    };
    let file_part = reqwest::multipart::Part::stream(reqwest::Body::wrap_stream(async_stream))
      .file_name(file_name.clone())
      .mime_str(&content_type)?;
    self.upload_file(file_part, query, auth).await
  }

  #[logfn(Info)]
  pub async fn download(
    &self,
    url_path: &str,
    auth: Option<(String, String)>,
  ) -> anyhow::Result<(StatusCode, ApiResponseResult<Vec<u8>>)> {
    let mut builder = self.client.get(format!("{}/{url_path}", self.addr));
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
  pub async fn download_into(
    &self,
    url_path: &str,
    auth: Option<(String, String)>,
    mut dest: PathBuf,
  ) -> anyhow::Result<(StatusCode, ApiResponseResult<()>)> {
    if dest.is_dir() {
      dest.push(
        url_path
          .split('/')
          .into_iter()
          .nth(1)
          .ok_or_else(|| anyhow::anyhow!("The url_path is invalid."))?,
      );
    }
    let url = format!("{}/{url_path}", self.addr);
    let mut builder = self.client.get(&url);
    if let Some((user, pass)) = auth {
      builder = builder.basic_auth(user, Some(pass));
    }
    let resp = builder.send().await?;
    let status = resp.status();
    if !status.is_success() {
      let error = resp.json::<BodyResponseError>().await?;
      return Ok((status, ApiResponseResult::Err(error)));
    }
    if let Some(parent) = dest.parent() {
      tokio::fs::create_dir_all(parent).await?;
    }
    let mut file = tokio::fs::File::create(dest).await?;
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
      let chunk = chunk?;
      file.write_all(&chunk).await?;
    }
    Ok((status, ApiResponseResult::Ok(())))
  }

  #[logfn(Info)]
  pub async fn download_with_progress_bar(
    &self,
    url_path: &str,
    auth: Option<(String, String)>,
    mut dest: PathBuf,
  ) -> anyhow::Result<(StatusCode, ApiResponseResult<()>)> {
    if dest.is_dir() {
      dest.push(
        url_path
          .split('/')
          .into_iter()
          .next()
          .ok_or_else(|| anyhow::anyhow!("The url_path is invalid."))?,
      );
    }
    let url = format!("{}/{url_path}", self.addr);
    let mut builder = self.client.get(&url);
    if let Some((user, pass)) = auth {
      builder = builder.basic_auth(user, Some(pass));
    }
    let resp = builder.send().await?;
    let status = resp.status();
    if !status.is_success() {
      let error = resp.json::<BodyResponseError>().await?;
      return Ok((status, ApiResponseResult::Err(error)));
    }
    let total_size = resp
      .content_length()
      .ok_or_else(|| anyhow::anyhow!("content length not found"))?;
    if let Some(parent) = dest.parent() {
      tokio::fs::create_dir_all(parent).await?;
    }
    let mut file = tokio::fs::File::create(dest).await?;
    let pb = progress_bar(total_size)?;
    let mut stream = resp.bytes_stream();
    let mut downloaded: u64 = 0;
    while let Some(chunk) = stream.next().await {
      let chunk = chunk?;
      file.write_all(&chunk).await?;
      downloaded += chunk.len() as u64;
      pb.set_position(downloaded.min(total_size));
    }
    pb.finish_with_message(format!(
      "The download of the file has been completed successfully."
    ));
    Ok((status, ApiResponseResult::Ok(())))
  }

  #[logfn(Info)]
  pub async fn info(
    &self,
    url_path: &str,
    auth: Option<(String, String)>,
  ) -> anyhow::Result<(StatusCode, ApiResponseResult<MetaDataFileResponse>)> {
    let mut builder = self.client.get(format!("{}/info/{url_path}", self.addr));
    if let Some((user, pass)) = auth {
      builder = builder.basic_auth(user, Some(pass));
    }
    let resp = builder.send().await?;
    Ok((resp.status(), resp.json().await?))
  }

  #[logfn(Info)]
  pub async fn delete(
    &self,
    url_path: &str,
    auth: Option<(String, String)>,
  ) -> anyhow::Result<(StatusCode, ApiResponseResult)> {
    let mut builder = self.client.delete(format!("{}/{url_path}", self.addr));
    if let Some((user, pass)) = auth {
      builder = builder.basic_auth(user, Some(pass));
    }
    let resp = builder.send().await?;
    Ok((resp.status(), resp.json().await?))
  }
}
