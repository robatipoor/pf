use api::{
  error::ApiResult,
  server::{worker::GarbageCollectorTask, ApiServer},
  util::{self, arg::get_env_source, tracing::INIT_SUBSCRIBER},
};
use clap::Parser;
use futures_util::FutureExt;
use once_cell::sync::Lazy;
use tracing::warn;

#[tokio::main]
async fn main() -> ApiResult {
  let args = api::configure::Args::parse();
  let config = api::configure::ApiConfig::read(args.settings, get_env_source("PF"))?;
  Lazy::force(&INIT_SUBSCRIBER);
  if let Err(e) = tokio::fs::create_dir_all(&config.fs.base_dir).await {
    warn!("Failed to create base directory, Error: {e}");
  };
  let server = ApiServer::new(config).await?;
  let gc_task = GarbageCollectorTask::new(server.state.clone());

  util::task::join_all(vec![
    ("HTTP_SERVER", true, server.run().boxed()),
    ("GC_TASK", true, gc_task.run().boxed()),
  ])
  .await?;
  Ok(())
}
