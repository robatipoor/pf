use crate::constant::ENV_PREFIX;
use config::Environment;
use once_cell::sync::Lazy;
use sdk::util::dir::get_cargo_project_root;
use serde::Deserialize;
use std::{
  net::{AddrParseError, SocketAddr},
  path::PathBuf,
};

use self::env::get_env_source;

pub mod args;
pub mod cors;
pub mod env;

pub static CONFIG: Lazy<ApiConfig> =
  Lazy::new(|| ApiConfig::read(None, get_env_source(ENV_PREFIX)).unwrap());

#[derive(Debug, Deserialize, Clone)]
pub struct ApiConfig {
  pub server: ServerConfig,
  pub fs: FileSystemConfig,
  pub db: DatabaseConfig,
  pub max_upload_bytes_size: usize,
  pub default_code_length: usize,
  pub default_expire_secs: u64,
  pub allow_manual_deletion: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
  domain_name: String,
  public_addr: Option<String>,
  pub schema: UrlSchema,
  host: String,
  pub port: u16,
  file_tls_key_path: Option<String>,
  file_tls_cert_path: Option<String>,
}

#[derive(Debug, Deserialize, Clone, strum::Display, Copy)]
pub enum UrlSchema {
  #[serde(rename = "http")]
  #[strum(serialize = "http")]
  Http,
  #[serde(rename = "https")]
  #[strum(serialize = "https")]
  Https,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
  pub path_dir: PathBuf,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FileSystemConfig {
  pub base_dir: PathBuf,
}

impl ServerConfig {
  pub fn get_http_addr(&self) -> String {
    format!("{}://{}:{}", self.schema, self.host, self.port)
  }

  pub fn get_domain_name(&self) -> String {
    format!("{}://{}", self.schema, self.domain_name)
  }

  pub fn get_public_addr(&self) -> Option<String> {
    self
      .public_addr
      .as_ref()
      .map(|addr| format!("{}://{addr}", self.schema))
  }

  pub fn get_socket_addr(&self) -> Result<SocketAddr, AddrParseError> {
    format!("{}:{}", self.host, self.port).parse()
  }

  pub fn get_tls_config(&self) -> anyhow::Result<tokio_rustls::rustls::ServerConfig> {
    crate::server::axum_tls::rustls_server_config(
      self.file_tls_key_path.as_ref().ok_or_else(|| {
        anyhow::anyhow!(
          "The `file_tls_key_path` setting should be configured in the settings file."
        )
      })?,
      self.file_tls_cert_path.as_ref().ok_or_else(|| {
        anyhow::anyhow!(
          "The `file_tls_cert_path` setting should be configured in the settings file."
        )
      })?,
    )
  }
}

impl ApiConfig {
  pub fn read(
    file_src: Option<PathBuf>,
    env_src: Environment,
  ) -> Result<Self, config::ConfigError> {
    config::Config::builder()
      .add_source(config::File::from(
        get_basic_settings_path(file_src)
          .map_err(|e| config::ConfigError::Message(e.to_string()))?,
      ))
      .add_source(env_src)
      .build()?
      .try_deserialize()
  }
}

fn get_basic_settings_path(file_src: Option<PathBuf>) -> std::io::Result<PathBuf> {
  if let Some(path) = file_src {
    Ok(path)
  } else {
    Ok(get_default_settings_dir()?.join("base.toml"))
  }
}

pub fn get_default_settings_dir() -> std::io::Result<PathBuf> {
  if let Some(root) = get_cargo_project_root()?.map(|root| root.join("api").join("settings")) {
    Ok(root)
  } else {
    std::env::current_dir()
  }
}

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn test_read_config() {
    ApiConfig::read(None, get_env_source("TEST_PF")).unwrap();
  }
}
