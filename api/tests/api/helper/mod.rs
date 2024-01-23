use std::ops::Deref;
use std::path::{Path, PathBuf};

use api::configure::CONFIG;
use api::server::{ApiServer, ApiState};
use api::unwrap;
use api::util::tracing::INIT_SUBSCRIBER;
use fake::{Fake, Faker};
use once_cell::sync::Lazy;
use sdk::client::PasteFileClient;
use sdk::model::request::UploadQueryParam;
use test_context::AsyncTestContext;

pub struct ApiTestContext {
  client: PasteFileClient,
  pub state: ApiState,
}

#[async_trait::async_trait]
impl AsyncTestContext for ApiTestContext {
  async fn setup() -> Self {
    Lazy::force(&INIT_SUBSCRIBER);
    let workspace = Path::new("test-dump").join(PathBuf::from(cuid2::create_id()));
    let db_path = Path::new("test-dump").join(PathBuf::from(cuid2::create_id()));
    tokio::fs::create_dir_all(&workspace).await.unwrap();
    let mut config = CONFIG.clone();
    config.server.port = 0;
    config.fs.base_dir = workspace;
    config.db.path = db_path;
    let server = ApiServer::new(config).await.unwrap();
    let state = server.state.clone();
    let client = PasteFileClient::new(server.state.config.server.get_http_addr());
    api::server::worker::spawn(axum::extract::State(state.clone()));
    tokio::spawn(server.run());
    Self { state, client }
  }

  async fn teardown(self) {
    tokio::fs::remove_dir_all(&self.state.config.db.path)
      .await
      .unwrap();
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
    auth: Option<(String, String)>,
  ) -> DummyFile {
    let file_name: String = format!("{}.txt", Faker.fake::<String>());
    let content_type = "text/plain";
    let content = Faker.fake::<String>().as_bytes().to_vec();
    let query = UploadQueryParam {
      max_download: max,
      code_length: len,
      expire_time: exp,
      delete_manually: del,
    };
    let (_, resp) = self
      .client
      .upload(
        file_name.clone(),
        content_type,
        &query,
        content.clone(),
        auth,
      )
      .await
      .unwrap();
    let resp = unwrap!(resp);
    let path = url::Url::parse(&resp.url).unwrap().path()[1..].to_string();
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
