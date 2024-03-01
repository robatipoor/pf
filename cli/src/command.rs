use sdk::{
  dto::{
    request::UploadQueryParam,
    response::{ApiResponseResult, BodyResponseError, MessageResponse, UploadResponse},
    FileUrlPath,
  },
  util::file::{add_extension, rm_extra_extension},
};
use std::path::{Path, PathBuf};
use url::Url;

use crate::{args::UploadOutput, client::CommandLineClient, util::crypto::KeyNonce};

pub struct UploadArguments {
  pub server_addr: String,
  pub auth: Option<(String, String)>,
  pub code_length: Option<usize>,
  pub progress_bar: bool,
  pub expire: Option<u64>,
  pub allow_manual_deletion: Option<bool>,
  pub max_download: Option<u32>,
  pub output: UploadOutput,
  pub source_file: PathBuf,
  pub key_nonce: Option<KeyNonce>,
}

pub struct CopyArguments {
  pub server_addr: String,
  pub auth: Option<(String, String)>,
  pub code_length: Option<usize>,
  pub expire: Option<u64>,
  pub allow_manual_deletion: Option<bool>,
  pub max_download: Option<u32>,
  pub output: UploadOutput,
  pub key_nonce: Option<KeyNonce>,
}

pub async fn ping(server_addr: String) {
  let client = CommandLineClient::new(server_addr);
  let (_, resp) = client.health_check().await.unwrap();
  match resp {
    ApiResponseResult::Ok(resp) => {
      println!("{}", serde_json::to_string(&resp).unwrap());
    }
    ApiResponseResult::Err(err) => print_response_err(&err),
  }
}

pub async fn upload(args: UploadArguments) {
  let mut source_file = args.source_file;
  if let Some(key_nonce) = args.key_nonce.as_ref() {
    source_file = crate::util::crypto::encrypt_upload_file(key_nonce, &source_file)
      .await
      .unwrap();
  }
  let param = UploadQueryParam {
    max_download: args.max_download,
    code_length: args.code_length,
    expire_secs: args.expire,
    allow_manual_deletion: args.allow_manual_deletion,
    qr_code_format: None,
  };
  let client = CommandLineClient::new(args.server_addr);
  let (_, resp) = if args.progress_bar {
    client
      .upload_with_progress_bar(&source_file, &param, args.auth)
      .await
  } else {
    client.upload_file(&source_file, &param, args.auth).await
  }
  .unwrap();
  show_upload_response(resp, args.output);
  if args.key_nonce.is_some() {
    tokio::fs::remove_file(source_file).await.unwrap();
  };
}

pub async fn copy(_args: CopyArguments) {}

fn show_upload_response(resp: ApiResponseResult<UploadResponse>, output: UploadOutput) {
  match resp {
    ApiResponseResult::Ok(resp) => match output {
      UploadOutput::Json => {
        println!("{}", serde_json::to_string(&resp).unwrap());
      }
      UploadOutput::QrCode => {
        let qr_code = sdk::util::qr_code::generate_text_qr_code(&resp.url).unwrap();
        println!("{qr_code}");
      }
      UploadOutput::Url => {
        println!("{}", resp.url);
      }
      UploadOutput::UrlPath => {
        println!("{}", &Url::parse(&resp.url).unwrap().path()[1..]);
      }
    },
    ApiResponseResult::Err(err) => print_response_err(&err),
  }
}

pub async fn download(
  server_addr: String,
  auth: Option<(String, String)>,
  progress_bar: bool,
  url_path: FileUrlPath,
  destination: PathBuf,
  key_nonce: Option<KeyNonce>,
) {
  let client = CommandLineClient::new(server_addr);
  let (_, resp) = if progress_bar {
    client
      .download_with_progress_bar(&url_path, auth, destination)
      .await
  } else {
    client.download_file(&url_path, auth, destination).await
  }
  .unwrap();
  match resp {
    ApiResponseResult::Ok(encrypt_source_file) => {
      if let Some(key_nonce) = key_nonce.as_ref() {
        crate::util::crypto::decrypt_download_file(key_nonce, &encrypt_source_file)
          .await
          .unwrap();
      }
      println!("{}", serde_json::to_string(&MessageResponse::ok()).unwrap());
    }
    ApiResponseResult::Err(err) => print_response_err(&err),
  }
}

pub async fn info(server_addr: String, url_path: FileUrlPath, auth: Option<(String, String)>) {
  let client = CommandLineClient::new(server_addr);
  let (_, resp) = client.info(&url_path, auth).await.unwrap();
  match resp {
    ApiResponseResult::Ok(resp) => {
      println!("{}", serde_json::to_string(&resp).unwrap());
    }
    ApiResponseResult::Err(err) => print_response_err(&err),
  }
}

pub async fn delete(server_addr: String, url_path: FileUrlPath, auth: Option<(String, String)>) {
  let client = CommandLineClient::new(server_addr);
  let (_, resp) = client.delete(&url_path, auth).await.unwrap();
  match resp {
    ApiResponseResult::Ok(resp) => {
      println!("{}", serde_json::to_string(&resp).unwrap());
    }
    ApiResponseResult::Err(err) => print_response_err(&err),
  }
}

pub async fn encrypt_file(key_nonce: &KeyNonce, source_file: &Path, mut destination: PathBuf) {
  if destination.is_dir() {
    destination = destination.join(add_extension(
      PathBuf::from(source_file.file_name().unwrap()),
      "bin",
    ));
  }
  crate::util::crypto::encrypt_file(key_nonce, source_file, destination)
    .await
    .unwrap();
}

pub async fn decrypt_file(key_nonce: &KeyNonce, source_file: &Path, mut destination: PathBuf) {
  if destination.is_dir() {
    destination = destination
      .join(rm_extra_extension(PathBuf::from(source_file.file_name().unwrap())).unwrap());
  }
  if source_file == destination {
    panic!("Please specify the valid destination file path.")
  }
  crate::util::crypto::decrypt_file(key_nonce, source_file, destination)
    .await
    .unwrap();
}

fn print_response_err(err: &BodyResponseError) {
  eprintln!("{}", serde_json::to_string(&err).unwrap());
  std::process::exit(1);
}
