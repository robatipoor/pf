use anyhow::anyhow;
use clap::{Parser, Subcommand};
use sdk::{client::PasteFileClient, model::request::UploadParamQuery, result::ApiResponseResult};
use std::{error::Error, path::PathBuf};
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
    deletable: bool,
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
      deletable,
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
        deletable: Some(deletable),
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
      let (_, resp) = client.download(&url.path()[1..], auth).await?;
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

  use std::process::Stdio;

  use assert_cmd::Command;
  use chrono::Utc;
  use fake::{Fake, Faker};
  use once_cell::sync::Lazy;
  use project_root::get_project_root;
  use sdk::model::response::{MessageResponse, MetaDataFileResponse, UploadResponse};
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
    async fn mock_delete_api(&self, file_name: &str) {
      let resp = success_delete_response();
      Mock::given(method("DELETE"))
        .and(path(&format!("/code/{file_name}")))
        .respond_with(resp)
        .mount(&self.server)
        .await;
    }

    async fn mock_info_api(&self, file_name: &str) {
      let resp = success_info_response();
      Mock::given(method("GET"))
        .and(path(&format!("/info/code/{file_name}")))
        .respond_with(resp)
        .mount(&self.server)
        .await;
    }

    async fn mock_upload_api(&self, file_name: &str) {
      let resp = success_upload_response();
      Mock::given(method("POST"))
        .and(path(format!("/upload/{file_name}")))
        .respond_with(resp)
        .mount(&self.server)
        .await;
    }
    async fn mock_download_api(&self, p: &str) {
      let resp = success_download_response();
      Mock::given(method("GET"))
        .and(path(p))
        .respond_with(resp)
        .mount(&self.server)
        .await;
    }
    // async fn mock_info_api(&self, file_name: &str) {
    //   let resp = success_info_response();
    //   Mock::given(method("GET"))
    //     .and(path(format!("/code/{file_name}")))
    //     .respond_with(resp)
    //     .mount(&self.server)
    //     .await;
    // }
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
    ctx.mock_upload_api(&ctx.upload_file).await;
    let _out = Command::cargo_bin("cli")
      .unwrap()
      .args([
        "--url",
        &ctx.server.uri(),
        "upload",
        "--file",
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
    ctx.mock_info_api(&file_name).await;
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
    let path = format!("{code}/{file_name}");
    ctx.mock_download_api(&path).await;
    let _out = Command::cargo_bin("cli")
      .unwrap()
      .args([
        "--url",
        &format!("{}/{path}", &ctx.server.uri()),
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
    let file_name: String = Faker.fake();
    ctx.mock_delete_api(&file_name).await;
    let _out = Command::cargo_bin("cli")
      .unwrap()
      .args([
        "--url",
        &format!("{}/code/{file_name}", &ctx.server.uri()),
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
      create_at: Utc::now(),
      expire_time: Utc::now(),
      is_deletable: true,
      max_download: None,
      downloads: 1,
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
    let body = r#"
--12345
Content-Disposition: form-data; name="file_name"

some text ...
--12345"#;
    ResponseTemplate::new(200).set_body_raw(body.as_bytes(), "multipart/form-data; boundary=12345")
  }
}
