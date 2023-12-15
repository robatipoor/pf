pub mod worker;

use crate::config::ApiConfig;
use crate::database::Database;
use crate::error::ApiResult;
use crate::router::get_router;
use std::sync::Arc;


#[derive(Clone)]
pub struct ApiState {
  pub config: Arc<ApiConfig>,
  pub db: Arc<Database>,
}

impl ApiState {
  pub fn new(config: ApiConfig) -> ApiResult<Self> {
    let db = Database::new(&config.db)?;
    Ok(Self {
      config: Arc::new(config),
      db: Arc::new(db),
    })
  }
}

pub struct ApiServer {
  pub state: ApiState,
  tcp: tokio::net::TcpListener,
}

impl ApiServer {
  pub async fn new(mut config: ApiConfig) -> ApiResult<Self> {
    let tcp = tokio::net::TcpListener::bind(config.server.get_socket_addr()?).await?;
    let addr = tcp.local_addr()?;
    tracing::info!("The server is listening on: {addr}");
    config.server.port = addr.port();
    let state = ApiState::new(config)?;
    Ok(Self { state, tcp })
  }

  pub async fn run(self) -> ApiResult<()> {
    let router = get_router(self.state);
    axum::serve(self.tcp, router).await?;
    Ok(())
  }
}

