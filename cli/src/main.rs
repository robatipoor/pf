use anyhow::anyhow;
use base64::Engine;
use clap::{Parser, Subcommand};
use sdk::{
  client::PasteFileClient, dto::request::UploadQueryParam, result::ApiResponseResult,
  util::base64::BASE64_ENGIN,
};
use std::{error::Error, path::PathBuf};
use url::Url;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
  #[arg(short, long)]
  server_addr: String,
  #[clap(short, long, value_parser = parse_auth)]
  auth: Option<(String, String)>,
  #[clap(subcommand)]
  cmd: SubCommand,
}

#[derive(Subcommand, Debug)]
pub enum SubCommand {
  Ping,
  Upload {
    #[clap(short, long)]
    code_length: Option<usize>,
    #[clap(short, long,value_parser = parse_expire_time_from_str)]
    expire: Option<u64>,
    #[clap(short, long)]
    max_download: Option<u32>,
    #[clap(short, long)]
    delete_manually: Option<bool>,
    #[clap(default_value_t = false, short, long)]
    qrcode: bool,
    #[clap(default_value_t = false, short, long)]
    progress_bar: bool,
    #[clap(short, long)]
    source_file: PathBuf,
  },
  Delete {
    #[arg(short, long)]
    url_path: String,
  },
  Info {
    #[arg(short, long)]
    url_path: String,
  },
  Download {
    #[clap(default_value_t = false, short, long)]
    progress_bar: bool,
    #[arg(short, long)]
    url_path: String,
    #[clap(short, long)]
    destination_dir: PathBuf,
  },
}

fn parse_expire_time_from_str(
  expire_time: &str,
) -> Result<u64, Box<dyn Error + Send + Sync + 'static>> {
  let words: Vec<&str> = expire_time.split_whitespace().collect();
  if words.len() != 2 {
    return Err("Invalid expire time format".into());
  }
  let value: u64 = words[0].parse()?;
  let multiplier = match words[1].to_lowercase().as_str() {
    "second" | "sec" | "s" => value,
    "minute" | "min" => value * 60,
    "hour" | "h" => value * 3600,
    "day" | "d" => value * 3600 * 24,
    "month" => value * 3600 * 24 * 30,
    "year" | "y" => value * 3600 * 24 * 30 * 12,
    _ => return Err("Invalid expire time format".into()),
  };
  Ok(value * multiplier)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let args = Args::parse();
  let client = PasteFileClient::new(args.server_addr);
  match args.cmd {
    SubCommand::Ping => {
      let (_, resp) = client.health_check().await?;
      match resp {
        ApiResponseResult::Ok(resp) => {
          println!("{}", resp.message);
        }
        ApiResponseResult::Err(err) => {
          return Err(anyhow!("{}", serde_json::to_string(&err)?));
        }
      }
    }
    SubCommand::Upload {
      code_length,
      progress_bar,
      expire,
      delete_manually,
      max_download,
      qrcode,
      source_file,
    } => {
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
      }?;
      match resp {
        ApiResponseResult::Ok(resp) => {
          if qrcode {
            println!(
              "{}",
              std::str::from_utf8(&BASE64_ENGIN.decode(resp.qrcode)?)?
            );
          } else {
            println!("{}", &Url::parse(&resp.url)?.path()[1..]);
          }
        }
        ApiResponseResult::Err(err) => {
          return Err(anyhow!("{}", serde_json::to_string(&err)?));
        }
      }
    }
    SubCommand::Download {
      progress_bar,
      url_path,
      destination_dir,
    } => {
      let (_, resp) = if progress_bar {
        client
          .download_with_progress_bar(&url_path, args.auth, &destination_dir)
          .await
      } else {
        client
          .download_into(&url_path, args.auth, &destination_dir)
          .await
      }?;
      match resp {
        ApiResponseResult::Ok(_) => {
          println!("Done");
        }
        ApiResponseResult::Err(err) => {
          return Err(anyhow!("{}", serde_json::to_string(&err)?));
        }
      }
    }
    SubCommand::Info { url_path } => {
      let (_, resp) = client.info(&url_path, args.auth).await?;
      match resp {
        ApiResponseResult::Ok(resp) => {
          println!("{}", serde_json::to_string(&resp)?);
        }
        ApiResponseResult::Err(err) => {
          return Err(anyhow!("{}", serde_json::to_string(&err)?));
        }
      }
    }
    SubCommand::Delete { url_path } => {
      let (_, resp) = client.delete(&url_path, args.auth).await?;
      match resp {
        ApiResponseResult::Ok(resp) => {
          println!("{}", serde_json::to_string(&resp)?);
        }
        ApiResponseResult::Err(err) => {
          return Err(anyhow!("{}", serde_json::to_string(&err)?));
        }
      }
    }
  }
  Ok(())
}

fn parse_auth(s: &str) -> Result<(String, String), Box<dyn Error + Send + Sync + 'static>> {
  let pos = s
    .find(':')
    .ok_or_else(|| format!("invalid username:password: no `:` found in {s}"))?;
  Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

// fn base_url(url: &url::Url) -> String {
//   format!(
//     "{}://{}:{}",
//     url.scheme(),
//     url.host_str().unwrap(),
//     url.port().unwrap()
//   )
// }

#[cfg(test)]
mod tests {

  use assert_cmd::Command;
  use chrono::Utc;
  use fake::{Fake, Faker};
  use once_cell::sync::Lazy;
  use project_root::get_project_root;
  use sdk::dto::response::{MessageResponse, MetaDataFileResponse, UploadResponse};
  use std::process::Stdio;
  use test_context::AsyncTestContext;
  use tokio::io::AsyncWriteExt;
  use tracing::info;
  use wiremock::matchers::{method, path};
  use wiremock::{Mock, MockServer, ResponseTemplate};

  static SETUP: Lazy<()> = Lazy::new(|| {
    tracing_subscriber::fmt().init();
    let root_dir = get_project_root().unwrap();
    std::process::Command::new("cargo")
      .arg("build")
      .arg("-q")
      .current_dir(&root_dir)
      .stdout(Stdio::piped())
      .spawn()
      .unwrap()
      .wait()
      .unwrap();
    info!("Success build project");
  });

  pub struct CliTestContext {
    pub server: MockServer,
    pub upload_file: String,
    pub download_dir: String,
  }

  impl CliTestContext {
    async fn new() -> Self {
      Lazy::force(&SETUP);
      let server = MockServer::start().await;
      let file_content: String = Faker.fake();
      let upload_file = format!("{}.txt", Faker.fake::<String>());
      let mut file = tokio::fs::File::create(&upload_file).await.unwrap();
      file.write_all(file_content.as_bytes()).await.unwrap();
      let download_dir = Faker.fake::<String>();
      tokio::fs::create_dir(&download_dir).await.unwrap();
      Self {
        server,
        upload_file,
        download_dir,
      }
    }
    async fn mock_ping_api(&self) {
      let success_ping = success_ping_response();
      Mock::given(method("GET"))
        .and(path("/healthz"))
        .respond_with(success_ping)
        .mount(&self.server)
        .await;
    }
    async fn mock_delete_api(&self, code: &str, file_name: &str) {
      let resp = success_delete_response();
      Mock::given(method("DELETE"))
        .and(path(&format!("/{code}/{file_name}")))
        .respond_with(resp)
        .mount(&self.server)
        .await;
    }
    async fn mock_info_api(&self, code: &str, file_name: &str) {
      let resp = success_info_response();
      Mock::given(method("GET"))
        .and(path(&format!("/info/{code}/{file_name}")))
        .respond_with(resp)
        .mount(&self.server)
        .await;
    }
    async fn mock_upload_api(&self) {
      let resp = success_upload_response();
      Mock::given(method("POST"))
        .and(path(format!("/upload")))
        .respond_with(resp)
        .mount(&self.server)
        .await;
    }
    async fn mock_download_api(&self, code: &str, file_name: &str) {
      let resp = success_download_response();
      Mock::given(method("GET"))
        .and(path(&format!("/{code}/{file_name}")))
        .respond_with(resp)
        .mount(&self.server)
        .await;
    }
  }

  #[async_trait::async_trait]
  impl AsyncTestContext for CliTestContext {
    async fn setup() -> Self {
      CliTestContext::new().await
    }

    async fn teardown(self) {
      tokio::fs::remove_file(self.upload_file).await.unwrap();
      tokio::fs::remove_dir_all(self.download_dir).await.unwrap();
    }
  }

  #[test_context::test_context(CliTestContext)]
  #[tokio::test]
  async fn test_upload_command(ctx: &mut CliTestContext) {
    ctx.mock_upload_api().await;
    let _out = Command::cargo_bin("cli")
      .unwrap()
      .args([
        "--url",
        &ctx.server.uri(),
        "upload",
        "--path",
        &ctx.upload_file,
      ])
      .assert()
      .success()
      .to_string();
  }

  #[test_context::test_context(CliTestContext)]
  #[tokio::test]
  async fn test_info_command(ctx: &mut CliTestContext) {
    let code: String = Faker.fake();
    let file_name: String = Faker.fake();
    ctx.mock_info_api(&code, &file_name).await;
    let _out = Command::cargo_bin("cli")
      .unwrap()
      .args([
        "--url",
        &format!("{}/{code}/{file_name}", &ctx.server.uri()),
        "info",
      ])
      .assert()
      .success()
      .to_string();
  }

  #[test_context::test_context(CliTestContext)]
  #[tokio::test]
  async fn test_download_command(ctx: &mut CliTestContext) {
    let code: String = Faker.fake();
    let file_name = "file.txt";
    ctx.mock_download_api(&code, &file_name).await;
    let _out = Command::cargo_bin("cli")
      .unwrap()
      .args([
        "--url",
        &format!("{}/{code}/{file_name}", &ctx.server.uri()),
        "download",
        "--path",
        &ctx.download_dir,
      ])
      .assert()
      .success()
      .to_string();
    // TODO try read file
  }

  #[test_context::test_context(CliTestContext)]
  #[tokio::test]
  async fn test_delete_command(ctx: &mut CliTestContext) {
    let code: String = Faker.fake();
    let file_name: String = Faker.fake();
    ctx.mock_delete_api(&code, &file_name).await;
    let _out = Command::cargo_bin("cli")
      .unwrap()
      .args([
        "--url",
        &format!("{}/{code}/{file_name}", &ctx.server.uri()),
        "delete",
      ])
      .assert()
      .success()
      .to_string();
  }

  #[test_context::test_context(CliTestContext)]
  #[tokio::test]
  async fn test_ping_command(ctx: &mut CliTestContext) {
    ctx.mock_ping_api().await;
    let _out = Command::cargo_bin("cli")
      .unwrap()
      .args(["--url", &ctx.server.uri(), "ping"])
      .assert()
      .success()
      .to_string();
  }

  fn success_ping_response() -> ResponseTemplate {
    let msg = MessageResponse {
      message: "OK".to_string(),
    };
    let body = serde_json::to_string(&msg).unwrap();
    ResponseTemplate::new(200).set_body_raw(body.as_bytes(), "application/json")
  }

  fn success_delete_response() -> ResponseTemplate {
    let msg = MessageResponse {
      message: "OK".to_string(),
    };
    let body = serde_json::to_string(&msg).unwrap();
    ResponseTemplate::new(200).set_body_raw(body.as_bytes(), "application/json")
  }

  fn success_info_response() -> ResponseTemplate {
    let msg = MetaDataFileResponse {
      created_at: Utc::now(),
      expiration_date: Utc::now(),
      delete_manually: true,
      max_download: None,
      count_downloads: 1,
    };
    let body = serde_json::to_string(&msg).unwrap();
    ResponseTemplate::new(200).set_body_raw(body.as_bytes(), "application/json")
  }

  fn success_upload_response() -> ResponseTemplate {
    let msg = UploadResponse {
      expire_time: Utc::now(),
      url: "test-address.url/code/file".to_string(),
      qrcode: "QR_CODE".to_string(),
    };
    let body = serde_json::to_string(&msg).unwrap();
    ResponseTemplate::new(200).set_body_raw(body.as_bytes(), "application/json")
  }

  fn success_download_response() -> ResponseTemplate {
    let body = "--12345\r\nContent-Disposition: form-data; name=\"my_text_field\"\r\n\r\nabcd\r\n--12345--\r\n";
    ResponseTemplate::new(200).set_body_raw(body.as_bytes(), "multipart/form-data; boundary=12345")
  }
}
