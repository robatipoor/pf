use api::{
  error::ApiResult,
  server::ApiServer,
  util::{arg::get_env_source, tracing::INIT_SUBSCRIBER},
};
use axum::extract::State;
use clap::Parser;
use once_cell::sync::Lazy;
use tracing::warn;

#[tokio::main]
async fn main() -> ApiResult {
  let args = api::configure::Args::parse();
  let config = api::configure::ApiConfig::read(args.settings, get_env_source("PF"))?;
  Lazy::force(&INIT_SUBSCRIBER);
  if let Err(e) = tokio::fs::create_dir_all(&config.fs.base_dir).await {
    warn!("Failed to create base directory: {e}");
  };
  let server = ApiServer::new(config).await?;
  api::server::worker::spawn(State(server.state.clone()));
  server.run().await?;
  Ok(())
}
