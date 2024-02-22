use std::path::{Path, PathBuf};

use base64::{engine::general_purpose::STANDARD, Engine};
use sdk::{
  dto::{
    request::UploadQueryParam,
    response::{ApiResponseResult, BodyResponseError, MessageResponse},
  },
  util::file::{add_extension, rm_extra_extension},
};
use url::Url;

use crate::{args::UploadOutput, client::CommandLineClient, util::crypto::KeyNonce};

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

pub struct UploadArguments {
  pub server_addr: String,
  pub auth: Option<(String, String)>,
  pub code_length: Option<usize>,
  pub progress_bar: bool,
  pub expire: Option<u64>,
  pub delete_manually: Option<bool>,
  pub max_download: Option<u32>,
  pub out: UploadOutput,
  pub source_file: PathBuf,
  pub key_nonce: Option<KeyNonce>,
}

pub async fn upload(args: UploadArguments) {
  let mut source_file = args.source_file;
  if let Some(key_nonce) = args.key_nonce.as_ref() {
    source_file = crate::util::crypto::encrypt_file(key_nonce, &source_file)
      .await
      .unwrap();
  }
  let query = UploadQueryParam {
    max_download: args.max_download,
    code_length: args.code_length,
    expire_secs: args.expire,
    delete_manually: args.delete_manually,
  };
  let client = CommandLineClient::new(args.server_addr);
  let (_, resp) = if args.progress_bar {
    client
      .upload_with_progress_bar(&source_file, &query, args.auth)
      .await
  } else {
    client.upload_from(&source_file, &query, args.auth).await
  }
  .unwrap();
  match resp {
    ApiResponseResult::Ok(resp) => match args.out {
      UploadOutput::Json => {
        println!("{}", serde_json::to_string(&resp).unwrap());
      }
      UploadOutput::QrCode => {
        println!(
          "{}",
          std::str::from_utf8(&STANDARD.decode(resp.qrcode).unwrap()).unwrap()
        );
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
  if args.key_nonce.is_some() {
    tokio::fs::remove_file(source_file).await.unwrap();
  };
}

pub async fn download(
  server_addr: String,
  auth: Option<(String, String)>,
  progress_bar: bool,
  url_path: String,
  destination: PathBuf,
  key_nonce: Option<KeyNonce>,
) {
  let client = CommandLineClient::new(server_addr);
  let (_, resp) = if progress_bar {
    client
      .download_with_progress_bar(&url_path, auth, destination)
      .await
  } else {
    client.download_into(&url_path, auth, destination).await
  }
  .unwrap();
  match resp {
    ApiResponseResult::Ok(encrypt_source_file) => {
      if let Some(key_nonce) = key_nonce.as_ref() {
        crate::util::crypto::decrypt_file(key_nonce, &encrypt_source_file)
          .await
          .unwrap();
      }
      println!("{}", serde_json::to_string(&MessageResponse::ok()).unwrap());
    }
    ApiResponseResult::Err(err) => print_response_err(&err),
  }
}

pub async fn info(server_addr: String, url_path: String, auth: Option<(String, String)>) {
  let client = CommandLineClient::new(server_addr);
  let (_, resp) = client.info(&url_path, auth).await.unwrap();
  match resp {
    ApiResponseResult::Ok(resp) => {
      println!("{}", serde_json::to_string(&resp).unwrap());
    }
    ApiResponseResult::Err(err) => print_response_err(&err),
  }
}

pub async fn delete(server_addr: String, url_path: String, auth: Option<(String, String)>) {
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
  crate::util::crypto::encrypt(key_nonce, source_file, &destination)
    .await
    .unwrap();
}

pub async fn decrypt_file(key_nonce: &KeyNonce, source_file: &Path, mut destination: PathBuf) {
  if destination.is_dir() {
    destination = destination
      .join(rm_extra_extension(PathBuf::from(source_file.file_name().unwrap())).unwrap());
  }
  if source_file == destination {
    panic!("Please specify the destination file path.")
  }
  crate::util::crypto::decrypt(key_nonce, source_file, &destination)
    .await
    .unwrap();
}

fn print_response_err(err: &BodyResponseError) {
  eprintln!("{}", serde_json::to_string(&err).unwrap());
  std::process::exit(1);
}
