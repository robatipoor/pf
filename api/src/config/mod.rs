use config::ConfigError;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::{
  net::{AddrParseError, SocketAddr},
  path::PathBuf,
};
use util::arg::{get_env_source, parse_config_path_from_arguments};

pub static CONFIG: Lazy<AppConfig> = Lazy::new(|| AppConfig::read().unwrap());

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
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
  pub addr: String,
  pub port: u16,
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
    format!("http://{}:{}", self.addr, self.port)
  }

  pub fn get_socket_addr(&self) -> Result<SocketAddr, AddrParseError> {
    format!("{}:{}", self.addr, self.port).parse()
  }
}

impl AppConfig {
  pub fn read() -> Result<Self, config::ConfigError> {
    let mut cfg = config::Config::builder();
    let env_src = get_env_source("PF");
    if let Some(path) = parse_config_path_from_arguments(&std::env::args().collect::<Vec<String>>())
    {
      cfg = cfg.add_source(config::File::from(path)).add_source(env_src);
    } else {
      let base_config = std::env::current_dir()
        .map_err(|e| ConfigError::Message(e.to_string()))?
        .join("config")
        .join("base.toml");
      cfg = cfg
        .add_source(config::File::from(base_config))
        .add_source(env_src);
    }
    cfg.build()?.try_deserialize()
  }
}

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn test_read_config() {
    let config = AppConfig::read();
    assert!(config.is_ok());
  }
}
