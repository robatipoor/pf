pub mod worker;

use crate::config::AppConfig;
use crate::database::Database;
use crate::router::get_router;
use axum::routing::IntoMakeService;
use axum::routing::Router;
use axum::Server;
use hyper::server::conn::AddrIncoming;
use sdk::error::ApiResult;
use std::net::TcpListener;
use std::sync::Arc;
use tracing::info;

#[derive(Clone)]
pub struct ApiState {
  pub config: Arc<AppConfig>,
  pub db: Arc<Database>,
  pub http: reqwest::Client,
}

pub struct ApiServer {
  pub state: ApiState,
  pub start: axum::Server<AddrIncoming, IntoMakeService<Router>>,
}
impl ApiServer {
  pub async fn build(mut config: AppConfig) -> ApiResult<Self> {
    let socket_addr = config.server.get_socket_addr()?;
    let tcp = TcpListener::bind(socket_addr)?;
    let addr = tcp.local_addr()?;
    info!("Listening to: {addr}");
    config.server.port = addr.port();
    let db = Database::new(&config.db)?;
    let state = ApiState {
      http: reqwest::Client::new(),
      config: Arc::new(config),
      db: Arc::new(db),
    };
    let router = get_router(state.clone());
    let axum_server = Server::from_tcp(tcp)?.serve(router.into_make_service());
    Ok(Self {
      state,
      start: axum_server,
    })
  }
}
