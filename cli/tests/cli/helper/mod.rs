use fake::{Fake, Faker};
use once_cell::sync::Lazy;
use pf_sdk::{
  dto::FileUrlPath,
  retry,
  util::{dir::get_cargo_project_root, random::generate_random_string},
};

use std::{
  io,
  path::{Path, PathBuf},
  process::{ExitStatus, Stdio},
  str::FromStr,
};
use test_context::AsyncTestContext;
use tracing::info;

static SETUP: Lazy<()> = Lazy::new(|| {
  tracing_subscriber::fmt().init();
  std::process::Command::new("cargo")
    .arg("build")
    .arg("-q")
    .current_dir(&get_cargo_project_root().unwrap().unwrap())
    .stdout(Stdio::piped())
    .spawn()
    .unwrap()
    .wait()
    .unwrap();
  info!("Success build project");
});

pub struct CliTestContext {
  server: tokio::process::Child,
  pub workspace: PathBuf,
  pub root_dir: PathBuf,
  pub server_addr: String,
}

impl CliTestContext {
  async fn new() -> Self {
    Lazy::force(&SETUP);

    let root_dir = get_cargo_project_root().unwrap().unwrap();
    let workspace = root_dir.join(Path::new("test-dump").join(PathBuf::from(cuid2::create_id())));
    tokio::fs::create_dir_all(&workspace).await.unwrap();
    let db_path = workspace.join(PathBuf::from(cuid2::create_id()));
    let port = find_free_port().await.unwrap();
    let server_addr = format!("http://127.0.0.1:{port}");

    let child = tokio::process::Command::new("target/debug/pf-api")
      .args(["--settings", "api/settings/base.toml"])
      .env("PF__SERVER__PORT", port.to_string())
      .env("PF__DB__PATH_DIR", db_path)
      .env("PF__FS__BASE_DIR", workspace.clone())
      .current_dir(&root_dir)
      .stdout(Stdio::piped())
      .spawn()
      .expect("Failed to spawn API server process");

    let _ = retry!(
      || async { ping(&root_dir, &server_addr).await },
      |r: &Result<ExitStatus, io::Error>| match r {
        Ok(code) => code.success(),
        Err(_) => false,
      }
    );

    Self {
      server: child,
      server_addr,
      root_dir,
      workspace,
    }
  }

  pub async fn create_dummy_file(&self) -> anyhow::Result<(PathBuf, String)> {
    let content = Faker.fake::<String>();
    let file_name = self
      .workspace
      .join(format!("{}.txt", Faker.fake::<String>()));
    tokio::fs::write(&file_name, &content).await?;
    Ok((file_name, content))
  }

  pub async fn upload_dummy_file(&self) -> anyhow::Result<(FileUrlPath, String)> {
    let (file, content) = self.create_dummy_file().await?;
    let output = tokio::process::Command::new("target/debug/pf-cli")
      .args([
        "--server-addr",
        &self.server_addr,
        "upload",
        "--source-file",
        file.to_str().unwrap(),
        "--output",
        "url-path",
      ])
      .current_dir(&self.root_dir)
      .output()
      .await?;
    let url_path = FileUrlPath::from_str(std::str::from_utf8(&output.stdout)?.trim())?;
    Ok((url_path, content))
  }
}

impl AsyncTestContext for CliTestContext {
  async fn setup() -> Self {
    CliTestContext::new().await
  }

  async fn teardown(mut self) {
    if let Err(err) = self.server.kill().await {
      tracing::error!("Failed to kill the server. Error: {}", err);
    }
    tokio::fs::remove_dir_all(self.workspace).await.unwrap();
  }
}

async fn find_free_port() -> anyhow::Result<u16> {
  Ok(
    tokio::net::TcpListener::bind("127.0.0.1:0")
      .await?
      .local_addr()?
      .port(),
  )
}

async fn ping(root_dir: &Path, server_addr: &str) -> Result<ExitStatus, io::Error> {
  tokio::process::Command::new("target/debug/pf-cli")
    .args(["--server-addr", server_addr, "ping"])
    .current_dir(root_dir)
    .stderr(Stdio::null())
    .spawn()?
    .wait()
    .await
}

pub fn generate_random_key_nonce() -> String {
  format!(
    "{}:{}",
    generate_random_string(32),
    generate_random_string(19)
  )
}
