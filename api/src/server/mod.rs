pub mod worker;

use crate::config::ApiConfig;
use crate::database::Database;
use crate::error::ApiResult;
use crate::router::get_router;
use axum::routing::IntoMakeService;
use axum::routing::Router;
use axum::Server;
use hyper::server::conn::AddrIncoming;
use std::net::TcpListener;
use std::sync::Arc;
use tracing::info;

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
  pub start: axum::Server<AddrIncoming, IntoMakeService<Router>>,
}
impl ApiServer {
  pub async fn build(mut config: ApiConfig) -> ApiResult<Self> {
    let socket_addr = config.server.get_socket_addr()?;
    let tcp = TcpListener::bind(socket_addr)?;
    let addr = tcp.local_addr()?;
    info!("Listening to: {addr}");
    config.server.port = addr.port();
    let state = ApiState::new(config)?;
    let router = get_router(state.clone());
    let axum_server = Server::from_tcp(tcp)?.serve(router.into_make_service());
    Ok(Self {
      state,
      start: axum_server,
    })
  }
}
