use crate::util::arg::get_env_source;
use clap::Parser;
use config::ConfigError;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::{
  net::{AddrParseError, SocketAddr},
  path::PathBuf,
};

pub static CONFIG: Lazy<ApiConfig> = Lazy::new(|| ApiConfig::read().unwrap());

#[derive(Debug, Deserialize, Clone)]
pub struct ApiConfig {
  pub server: ServerConfig,
  pub fs: FileSystemConfig,
  pub db: DatabaseConfig,
  pub domain: String,
  pub max_upload_size: usize,
  pub default_code_length: usize,
  pub default_expire_time: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
  pub schema: UrlSchema,
  pub addr: String,
  pub port: u16,
}

#[derive(Debug, Deserialize, Clone,strum::Display)]
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
    format!("{}://{}:{}", self.schema,self.addr, self.port)
  }

  pub fn get_socket_addr(&self) -> Result<SocketAddr, AddrParseError> {
    format!("{}:{}", self.addr, self.port).parse()
  }
}

impl ApiConfig {
  pub fn read() -> Result<Self, config::ConfigError> {
    let mut cfg = config::Config::builder();
    let env_src = get_env_source("PF");
    if let Some(path) = Args::parse().config {
      cfg = cfg.add_source(config::File::from(path)).add_source(env_src);
    } else {
      let base_config = std::env::current_dir()
        .map_err(|e| ConfigError::Message(e.to_string()))?
        .join("settings")
        .join("base.toml");
      cfg = cfg
        .add_source(config::File::from(base_config))
        .add_source(env_src);
    }
    cfg.build()?.try_deserialize()
  }
}

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
  #[arg(short, long)]
  config: Option<PathBuf>,
}

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn test_read_config() {
    let config = ApiConfig::read();
    assert!(config.is_ok());
  }
}
