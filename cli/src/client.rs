use std::{
  ops::Deref,
  path::{Path, PathBuf},
};

use sdk::{
  client::PasteFileClient,
  dto::{
    request::UploadQueryParam,
    response::{ApiResponseResult, BodyResponseError, UploadResponse},
    FileUrlPath,
  },
};

use futures_util::StreamExt;
use reqwest::StatusCode;
use tokio::io::AsyncWriteExt;

use crate::util::progress::progress_bar;

pub struct CommandLineClient {
  pub inner: PasteFileClient,
}

impl CommandLineClient {
  pub fn new(addr: String) -> Self {
    Self {
      inner: PasteFileClient::new(addr),
    }
  }

  pub async fn upload_with_progress_bar(
    &self,
    source: &Path,
    param: &UploadQueryParam,
    auth: Option<(String, String)>,
  ) -> anyhow::Result<(StatusCode, ApiResponseResult<UploadResponse>)> {
    let file_name = sdk::util::file::get_file_name(source)?;
    let content_type = sdk::util::file::get_content_type(source)?;
    let file = tokio::fs::File::open(source).await?;
    let total_size = file.metadata().await?.len();
    let pb = progress_bar(total_size)?;
    self
      .upload_from_reader(
        file_name,
        &content_type,
        pb.wrap_async_read(file),
        param,
        auth,
      )
      .await
  }

  pub async fn download_with_progress_bar(
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
    let total_size = resp
      .content_length()
      .ok_or_else(|| anyhow::anyhow!("content length not found"))?;
    if let Some(parent) = destination.parent() {
      tokio::fs::create_dir_all(parent).await?;
    }
    let mut file = tokio::fs::File::create(&destination).await?;
    let pb = progress_bar(total_size)?;
    let mut stream = resp.bytes_stream();
    let mut downloaded: u64 = 0;
    while let Some(chunk) = stream.next().await {
      let chunk = chunk?;
      file.write_all(&chunk).await?;
      downloaded += chunk.len() as u64;
      pb.set_position(downloaded.min(total_size));
    }
    pb.finish_with_message("Download completed successfully.");
    Ok((status, ApiResponseResult::Ok(destination)))
  }
}

impl Deref for CommandLineClient {
  type Target = PasteFileClient;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}
