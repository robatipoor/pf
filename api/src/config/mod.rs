use common::config::{arg::parse_config_path_from_arguments, get_env_source};
use config::ConfigError;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::net::{AddrParseError, SocketAddr};

pub static CONFIG: Lazy<AppConfig> = Lazy::new(|| AppConfig::read().unwrap());

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
  pub server: ServerConfig,
  pub worker: WorkerConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
  pub addr: String,
  pub port: u16,
}

impl ServerConfig {
  pub fn get_http_addr(&self) -> String {
    format!("http://{}:{}", self.addr, self.port)
  }

  pub fn get_socket_addr(&self) -> Result<SocketAddr, AddrParseError> {
    format!("{}:{}", self.addr, self.port).parse()
  }
}

#[derive(Debug, Deserialize, Clone)]
pub struct WorkerConfig {
  pub failed_task_delay: u64,
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