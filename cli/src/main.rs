use args::{Args, SubCommand, UploadOutput};
use base64::{engine::general_purpose::STANDARD, Engine};
use clap::Parser;
use client::CommandLineClient;

use sdk::dto::{
  request::UploadQueryParam,
  response::{ApiResponseResult, BodyResponseError, MessageResponse},
};
use url::Url;
use util::crypto::{decrypt_file, encrypt_file};

mod args;
mod client;
mod parse;
mod util;

#[tokio::main]
async fn main() {
  let args = Args::parse();
  let client = CommandLineClient::new(args.server_addr);
  match args.cmd {
    SubCommand::Ping => {
      let (_, resp) = client.health_check().await.unwrap();
      match resp {
        ApiResponseResult::Ok(resp) => {
          println!("{}", serde_json::to_string(&resp).unwrap());
        }
        ApiResponseResult::Err(err) => print_response_err(&err),
      }
    }
    SubCommand::Upload {
      code_length,
      progress_bar,
      expire,
      delete_manually,
      max_download,
      out,
      mut source_file,
      encrypt,
    } => {
      if source_file.is_dir() {
        eprintln!("The source file option should be set to the path file.");
        std::process::exit(1);
      }
      if let Some(encrypt) = encrypt.as_ref() {
        source_file = encrypt_file(encrypt, &source_file).await.unwrap();
      }
      let query = UploadQueryParam {
        max_download,
        code_length,
        expire_secs: expire,
        delete_manually,
      };
      let (_, resp) = if progress_bar {
        client
          .upload_with_progress_bar(&source_file, &query, args.auth)
          .await
      } else {
        client.upload_from(&source_file, &query, args.auth).await
      }
      .unwrap();
      match resp {
        ApiResponseResult::Ok(resp) => match out {
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
      if encrypt.is_some() {
        // tokio::fs::remove_file(source_file).await.unwrap();
      }
    }
    SubCommand::Download {
      progress_bar,
      url_path,
      destination,
      decrypt,
    } => {
      let (_, resp) = if progress_bar {
        client
          .download_with_progress_bar(&url_path, args.auth, destination)
          .await
      } else {
        client
          .download_into(&url_path, args.auth, destination)
          .await
      }
      .unwrap();
      match resp {
        ApiResponseResult::Ok(encrypt_source_file) => {
          if let Some(decrypt) = decrypt.as_ref() {
            decrypt_file(decrypt, &encrypt_source_file).await.unwrap();
          }
          println!("{}", serde_json::to_string(&MessageResponse::ok()).unwrap());
        }
        ApiResponseResult::Err(err) => print_response_err(&err),
      }
    }
    SubCommand::Info { url_path } => {
      let (_, resp) = client.info(&url_path, args.auth).await.unwrap();
      match resp {
        ApiResponseResult::Ok(resp) => {
          println!("{}", serde_json::to_string(&resp).unwrap());
        }
        ApiResponseResult::Err(err) => print_response_err(&err),
      }
    }
    SubCommand::Delete { url_path } => {
      let (_, resp) = client.delete(&url_path, args.auth).await.unwrap();
      match resp {
        ApiResponseResult::Ok(resp) => {
          println!("{}", serde_json::to_string(&resp).unwrap());
        }
        ApiResponseResult::Err(err) => print_response_err(&err),
      }
    }
  };
}

fn print_response_err(err: &BodyResponseError) {
  eprintln!("{}", serde_json::to_string(&err).unwrap());
  std::process::exit(1);
}
