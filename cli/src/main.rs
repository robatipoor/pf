use anyhow::anyhow;
use assert_cmd::Command;
use clap::{Parser, Subcommand};
use sdk::model::response::MessageResponse;
use sdk::{client::PasteFileClient, model::request::UploadParamQuery, result::ApiResponseResult};
use std::{collections::HashMap, error::Error, path::PathBuf};
use test_context::{test_context, AsyncTestContext};
use tokio::io::AsyncWriteExt;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
  #[arg(short, long)]
  url: String,

  #[clap(subcommand)]
  cmd: SubCommand,
}

#[derive(Subcommand, Debug)]
pub enum SubCommand {
  Ping,
  Upload {
    #[clap(short, long, value_parser = parse_key_val::<String, String>)]
    auth: Option<(String, String)>,
    #[clap(default_value_t = 4, short, long)]
    code_length: usize,
    #[clap(default_value_t = 7200, short, long)]
    expire_time: u64,
    #[clap(short, long)]
    max_download: Option<u32>,
    #[clap(default_value_t = true, short, long)]
    deleteable: bool,
    #[clap(short, long)]
    file: PathBuf,
  },
  Delete {
    #[clap(short, long, value_parser = parse_key_val::<String, String>)]
    auth: Option<(String, String)>,
  },
  Info {
    #[clap(short, long, value_parser = parse_key_val::<String, String>)]
    auth: Option<(String, String)>,
  },
  Download {
    #[clap(short, long, value_parser = parse_key_val::<String, String>)]
    auth: Option<(String, String)>,
    #[clap(short, long)]
    path: PathBuf,
  },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let args = Args::parse();
  let url = url::Url::parse(&args.url)?;
  let client = PasteFileClient::new(&base_url(&url));
  match args.cmd {
    SubCommand::Ping => {
      let (_, resp) = client.health_check().await?;
      match resp {
        ApiResponseResult::Ok(resp) => {
          println!("{}", serde_json::to_string(&resp)?);
        }
        ApiResponseResult::Err(err) => {
          return Err(anyhow!("{}", serde_json::to_string(&err)?));
        }
      }
    }
    SubCommand::Upload {
      auth,
      code_length,
      expire_time,
      deleteable,
      max_download,
      file,
    } => {
      let file_name = file.file_name().unwrap().to_str().unwrap().to_string();
      let content_type = mime_guess::from_path(&file)
        .first()
        .unwrap()
        .essence_str()
        .to_owned();
      let file = tokio::fs::read(file).await?;
      let query = UploadParamQuery {
        max_download,
        code_length: Some(code_length),
        expire_time: Some(expire_time),
        deleteable: Some(deleteable),
      };
      let (_, resp) = client
        .upload(file_name, &content_type, &query, file, auth)
        .await?;
      match resp {
        ApiResponseResult::Ok(resp) => {
          println!("{}", serde_json::to_string(&resp)?);
        }
        ApiResponseResult::Err(err) => {
          return Err(anyhow!("{}", serde_json::to_string(&err)?));
        }
      }
    }
    SubCommand::Delete { auth } => {
      let (_, resp) = client.delete(&url.path()[1..], auth).await?;
      match resp {
        ApiResponseResult::Ok(resp) => {
          println!("{}", serde_json::to_string(&resp)?);
        }
        ApiResponseResult::Err(err) => {
          return Err(anyhow!("{}", serde_json::to_string(&err)?));
        }
      }
    }
    SubCommand::Info { auth } => {
      let (_, resp) = client.info(url.path(), auth).await?;
      match resp {
        ApiResponseResult::Ok(resp) => {
          println!("{}", serde_json::to_string(&resp)?);
        }
        ApiResponseResult::Err(err) => {
          return Err(anyhow!("{}", serde_json::to_string(&err)?));
        }
      }
    }
    SubCommand::Download { path, auth } => {
      let (_, resp) = client.download(url.path(), auth).await?;
      match resp {
        ApiResponseResult::Ok(resp) => {
          let file_name = PathBuf::from(url.path())
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
          let _ = tokio::fs::create_dir_all(&path).await;
          let mut file = tokio::fs::File::create(path.join(file_name)).await?;
          file.write_all(&resp).await?;
          println!("Ok");
        }
        ApiResponseResult::Err(err) => {
          return Err(anyhow!("{}", serde_json::to_string(&err)?));
        }
      }
    }
  }
  Ok(())
}

fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
  T: std::str::FromStr,
  T::Err: Error + Send + Sync + 'static,
  U: std::str::FromStr,
  U::Err: Error + Send + Sync + 'static,
{
  let pos = s
    .find(':')
    .ok_or_else(|| format!("invalid username:password: no `:` found in `{s}`"))?;
  Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

fn base_url(url: &url::Url) -> String {
  format!(
    "{}://{}:{}",
    url.scheme(),
    url.host_str().unwrap(),
    url.port().unwrap()
  )
}

#[cfg(test)]
mod tests {

  use assert_cmd::Command;
  use sdk::model::response::MessageResponse;
  use wiremock::matchers::{method, path};
  use wiremock::{Mock, MockServer, ResponseTemplate};

  pub struct CliTestContext {
    pub server: MockServer,
  }

  impl CliTestContext {
    async fn new() -> Self {
      let server = MockServer::start().await;
      Self { server }
    }
    async fn mock_ping_api(&self) {
      let success_ping = success_ping_response();
      Mock::given(method("GET"))
        .and(path("/healthz"))
        .respond_with(success_ping)
        .mount(&self.server)
        .await;
    }
    async fn mock_delete_api(&self) {
      let resp = success_delete_response();
      Mock::given(method("DELETE"))
        .and(path("/code/file"))
        .respond_with(resp)
        .mount(&self.server)
        .await;
    }
    async fn mock_upload_api(&self) {
      let resp = success_delete_response();
      Mock::given(method("POST"))
        .and(path("/code/file"))
        .respond_with(resp)
        .mount(&self.server)
        .await;
    }
    async fn mock_download_api(&self) {
      let resp = success_delete_response();
      Mock::given(method("GET"))
        .and(path("/code/file"))
        .respond_with(resp)
        .mount(&self.server)
        .await;
    }
    async fn mock_info_api(&self) {
      let resp = success_delete_response();
      Mock::given(method("GET"))
        .and(path("/code/file"))
        .respond_with(resp)
        .mount(&self.server)
        .await;
    }
  }

  #[tokio::test]
  async fn test_upload_command() {
    let ctx = CliTestContext::new().await;
    ctx.mock_delete_api().await;
    let out = Command::cargo_bin("cli")
      .unwrap()
      .args([
        "--url",
        &format!("{}/code/file", &ctx.server.uri()),
        "upload",
      ])
      .assert()
      .success()
      .to_string();
  }

  #[tokio::test]
  async fn test_info_command() {
    let ctx = CliTestContext::new().await;
    ctx.mock_info_api().await;
    let out = Command::cargo_bin("cli")
      .unwrap()
      .args([
        "--url",
        &format!("{}/code/file", &ctx.server.uri()),
        "upload",
      ])
      .assert()
      .success()
      .to_string();
  }

  #[tokio::test]
  async fn test_download_command() {
    let ctx = CliTestContext::new().await;
    ctx.mock_download_api().await;
    let out = Command::cargo_bin("cli")
      .unwrap()
      .args([
        "--url",
        &format!("{}/code/file", &ctx.server.uri()),
        "upload",
      ])
      .assert()
      .success()
      .to_string();
  }

  #[tokio::test]
  async fn test_ping_command() {
    let ctx = CliTestContext::new().await;
    ctx.mock_ping_api().await;
    let out = Command::cargo_bin("cli")
      .unwrap()
      .args(["--url", &ctx.server.uri(), "ping"])
      .assert()
      .success()
      .to_string();
  }

  #[tokio::test]
  async fn test_delete_command() {
    let ctx = CliTestContext::new().await;
    ctx.mock_delete_api().await;
    let out = Command::cargo_bin("cli")
      .unwrap()
      .args([
        "--url",
        &format!("{}/code/file", &ctx.server.uri()),
        "delete",
      ])
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
}
