use std::ops::Deref;

use api::config::CONFIG;
use api::server::{ApiServer, ApiState};
use common::client::PasteFileClient;
use common::config::tracing::INIT_SUBSCRIBER;
use common::model::request::UploadParamQuery;
use fake::{Fake, Faker};
use once_cell::sync::Lazy;
use test_context::AsyncTestContext;

pub struct ApiTestContext {
  client: PasteFileClient,
  pub state: ApiState,
}

#[async_trait::async_trait]
impl AsyncTestContext for ApiTestContext {
  async fn setup() -> Self {
    Lazy::force(&INIT_SUBSCRIBER);
    let workspace = std::path::PathBuf::from(cuid2::create_id());
    tokio::fs::create_dir_all(&workspace).await.unwrap();
    let mut config = CONFIG.clone();
    config.server.port = 0;
    config.fs.base_dir = workspace;
    let server = ApiServer::build(config).await.unwrap();
    let client = PasteFileClient::new(&server.state.config.server.get_http_addr());
    tokio::spawn(server.start);
    Self {
      state: server.state,
      client,
    }
  }

  async fn teardown(self) {
    tokio::fs::remove_dir_all(&self.state.config.fs.base_dir)
      .await
      .unwrap();
  }
}

impl Deref for ApiTestContext {
  type Target = PasteFileClient;

  fn deref(&self) -> &Self::Target {
    &self.client
  }
}

impl ApiTestContext {
  pub async fn upload_dummy_file(
    &self,
    max: Option<u32>,
    len: Option<usize>,
    exp: Option<u64>,
    del: Option<bool>,
  ) -> DummyFile {
    let file_name: String = format!("{}.txt", Faker.fake::<String>());
    let content_type = "text/plain";
    let content = Faker.fake::<String>().as_bytes().to_vec();
    let query = UploadParamQuery {
      max_download: max,
      length_code: len,
      expire_time: exp,
      deleteable: del,
    };
    let (status, resp) = self
      .client
      .upload(file_name.clone(), content_type, &query, content.clone())
      .await
      .unwrap();
    assert!(status.is_success());
    let path = url::Url::parse(&resp.unwrap().url).unwrap().path()[1..].to_string();
    DummyFile {
      content,
      content_type: content_type.to_string(),
      file_name,
      path,
    }
  }
}

#[derive(Clone)]
pub struct DummyFile {
  pub content: Vec<u8>,
  pub content_type: String,
  pub file_name: String,
  pub path: String,
}
