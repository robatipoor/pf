use api::{
  error::ApiResult,
  server::{worker::GarbageCollectorTask, ApiServer},
  util::{self, arg::get_env_source, tracing::INIT_SUBSCRIBER},
};
use clap::Parser;
use futures_util::FutureExt;
use once_cell::sync::Lazy;

#[tokio::main]
async fn main() -> ApiResult {
  // Parse command-line arguments
  let args = api::configure::Args::parse();
  // Read API configuration
  let config = api::configure::ApiConfig::read(args.settings, get_env_source("PF"))?;
  // Force initialization of subscriber
  Lazy::force(&INIT_SUBSCRIBER);
  // Create base directory if it doesn't exist
  tokio::fs::create_dir_all(&config.fs.base_dir).await?;
  // Initialize API server
  let server = ApiServer::new(config).await?;
  // Create garbage collector task
  let gc_task = GarbageCollectorTask::new(server.state.clone());
  // Start HTTP server and garbage collector task concurrently
  util::task::join_all(vec![
    ("HTTP_SERVER", true, server.run().boxed()),
    ("GC_TASK", true, gc_task.run().boxed()),
  ])
  .await?;
  Ok(())
}
