use std::ops::Deref;

use api::config::CONFIG;
use api::server::{ApiServer, ApiState};
use common::client::PasteFileClient;
use common::config::tracing::INIT_SUBSCRIBER;
use once_cell::sync::Lazy;
use test_context::AsyncTestContext;

pub mod dummy;

pub struct ApiTestContext {
  pub client: PasteFileClient,
  pub state: ApiState,
}

#[async_trait::async_trait]
impl AsyncTestContext for ApiTestContext {
  async fn setup() -> Self {
    Lazy::force(&INIT_SUBSCRIBER);
    let mut config = CONFIG.clone();
    config.server.port = 0;
    // config.db.path = PathBuf::from(cuid2::create_id());
    let server = ApiServer::build(config).await.unwrap();
    let client = PasteFileClient::new(&server.state.config.server.get_http_addr());
    tokio::spawn(server.start);
    Self {
      state: server.state,
      client,
    }
  }

  async fn teardown(self) {
    // tokio::fs::remove_dir_all(&self.state.config.db.path)
    //   .await
    //   .unwrap();
  }
}

impl Deref for ApiTestContext {
  type Target = PasteFileClient;

  fn deref(&self) -> &Self::Target {
    &self.client
  }
}
