use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;
use std::time::Duration;

use crate::config::AppConfig;
use crate::router::get_router;
use axum::routing::IntoMakeService;
use axum::routing::Router;
use axum::Server;
use common::error::ApiResult;
use hyper::server::conn::AddrIncoming;
use tokio::sync::RwLock;
use tracing::info;
// code/filename.ext
pub type PathFile = String;

pub type DataBase = Arc<RwLock<HashMap<PathFile, MetaDataFile>>>;

pub struct Sequential {
  inner: AtomicU32,
}

impl Sequential {
  pub fn new(min: u32) -> Self {
    Self {
      inner: AtomicU32::new(min),
    }
  }

  pub fn get_id(&self) -> Option<u32> {
    Some(self.inner.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
  }
}

#[derive(Clone)]
pub struct MetaDataFile {
  pub id: u32,
  pub expire_time: Duration,
  pub is_deleteable: bool,
  pub max_download: u16,
}

#[derive(Clone)]
pub struct ApiState {
  pub config: Arc<AppConfig>,
  pub http: Arc<reqwest::Client>,
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
    info!("Listening to: {addr} ...");
    config.server.port = addr.port();
    let state = ApiState {
      http: Arc::new(reqwest::Client::new()),
      config: Arc::new(config),
    };
    let router = get_router(state.clone());
    let axum_server = Server::from_tcp(tcp)?.serve(router.into_make_service());
    Ok(Self {
      state,
      start: axum_server,
    })
  }
}
