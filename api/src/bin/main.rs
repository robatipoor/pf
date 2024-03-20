use clap::Parser;
use futures_util::FutureExt;
use once_cell::sync::Lazy;
use pf_api::{
  configure::env::get_env_source,
  constant::ENV_PREFIX,
  error::result::ApiResult,
  server::{worker::GarbageCollectorTask, ApiServer},
  util::{self, tracing::INIT_SUBSCRIBER},
};

#[tokio::main]
async fn main() -> ApiResult {
  // Parse command-line arguments
  let args = pf_api::configure::args::Args::parse();
  // Read API configuration
  let config = pf_api::configure::ApiConfig::read(args.settings, get_env_source(ENV_PREFIX))?;
  // Validate settings
  config.validate()?;
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
    ("web server", true, server.run().boxed()),
    ("garbage collector", true, gc_task.run().boxed()),
  ])
  .await?;
  Ok(())
}
