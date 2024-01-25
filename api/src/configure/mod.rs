use crate::util::arg::get_env_source;
use config::Environment;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::{
  net::{AddrParseError, SocketAddr},
  path::PathBuf,
};

pub static CONFIG: Lazy<ApiConfig> =
  Lazy::new(|| ApiConfig::read(None, get_env_source("PF")).unwrap());

#[derive(Debug, Deserialize, Clone)]
pub struct ApiConfig {
  pub server: ServerConfig,
  pub fs: FileSystemConfig,
  pub db: DatabaseConfig,
  pub domain: String,
  pub max_upload_size: usize,
  pub default_code_length: usize,
  pub default_expire_secs: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
  pub schema: UrlSchema,
  pub host: String,
  pub port: u16,
}

#[derive(Debug, Deserialize, Clone, strum::Display)]
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
  pub path: PathBuf,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FileSystemConfig {
  pub base_dir: PathBuf,
}

impl ServerConfig {
  pub fn get_http_addr(&self) -> String {
    format!("{}://{}:{}", self.schema, self.host, self.port)
  }

  pub fn get_socket_addr(&self) -> Result<SocketAddr, AddrParseError> {
    format!("{}:{}", self.host, self.port).parse()
  }
}

impl ApiConfig {
  pub fn read(
    arg_file_src: Option<PathBuf>,
    env_src: Environment,
  ) -> Result<Self, config::ConfigError> {
    config::Config::builder()
      .add_source(config::File::from(
        get_basic_settings_path(arg_file_src)
          .map_err(|e| config::ConfigError::Message(e.to_string()))?,
      ))
      .add_source(env_src)
      .build()?
      .try_deserialize()
  }
}

fn get_basic_settings_path(arg_path: Option<PathBuf>) -> std::io::Result<PathBuf> {
  if let Some(path) = arg_path {
    Ok(path)
  } else {
    Ok(std::env::current_dir()?.join("settings").join("base.toml"))
  }
}

#[derive(clap::Parser, Debug, Default)]
#[command(author, version, about, long_about = None)]
pub struct Args {
  #[arg(short, long)]
  pub settings: Option<PathBuf>,
}

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn test_read_config() {
    let env_ns = "TEST_PF";
    ApiConfig::read(None, get_env_source(env_ns)).unwrap();
  }
}
