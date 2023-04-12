use api::server::ApiServer;
use axum::extract::State;
use common::{config::tracing::INIT_SUBSCRIBER, error::ApiResult};
use once_cell::sync::Lazy;

#[tokio::main]
async fn main() -> ApiResult {
  let config = api::config::AppConfig::read()?;
  Lazy::force(&INIT_SUBSCRIBER);
  let server = ApiServer::build(config).await?;
  api::server::worker::spawn(State(server.state.clone()));
  let _ = server.start.await;
  Ok(())
}
