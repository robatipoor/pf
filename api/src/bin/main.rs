use api::{error::ApiResult, server::ApiServer, util::tracing::INIT_SUBSCRIBER};
use axum::extract::State;
use once_cell::sync::Lazy;
use tracing::warn;

#[tokio::main]
async fn main() -> ApiResult {
  let config = api::config::ApiConfig::read()?;
  Lazy::force(&INIT_SUBSCRIBER);
  if let Err(e) = tokio::fs::create_dir_all(&config.fs.base_dir).await {
    warn!("failed create base dir: {e}");
  };
  let server = ApiServer::build(config).await?;
  api::server::worker::spawn(State(server.state.clone()));
  let _ = server.start.await;
  Ok(())
}
