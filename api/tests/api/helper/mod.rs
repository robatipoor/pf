use std::ops::Deref;
use std::path::{Path, PathBuf};

use crate::unwrap;
use api::configure::CONFIG;
use api::error::result::ApiResult;
use api::server::worker::GarbageCollectorTask;
use api::server::{ApiServer, ApiState};
use api::util::tracing::INIT_SUBSCRIBER;
use fake::{Fake, Faker};
use once_cell::sync::Lazy;
use sdk::client::PasteFileClient;
use sdk::dto::request::{QrCodeFormat, UploadQueryParam};
use sdk::dto::FileUrlPath;
use test_context::AsyncTestContext;

pub mod assert;

pub struct ApiTestContext {
  pub state: ApiState,
  pub workspace: PathBuf,
  client: PasteFileClient,
  server_task: tokio::task::JoinHandle<ApiResult>,
  gc_task: tokio::task::JoinHandle<ApiResult>,
}

#[async_trait::async_trait]
impl AsyncTestContext for ApiTestContext {
  async fn setup() -> Self {
    Lazy::force(&INIT_SUBSCRIBER);
    let workspace = Path::new("test-dump").join(PathBuf::from(cuid2::create_id()));
    tokio::fs::create_dir_all(&workspace).await.unwrap();
    let mut config = CONFIG.clone();
    config.server.port = 0;
    config.db.path_dir = workspace.join(PathBuf::from(cuid2::create_id()));
    config.fs.base_dir = workspace.clone();
    let server = ApiServer::new(config).await.unwrap();
    let state = server.state.clone();
    let client = PasteFileClient::new(server.state.config.server.get_http_addr());
    let gc_task = tokio::task::spawn(GarbageCollectorTask::new(state.clone()).run());
    let server_task = tokio::task::spawn(server.run());
    Self {
      state,
      client,
      workspace,
      server_task,
      gc_task,
    }
  }

  async fn teardown(self) {
    self.gc_task.abort();
    self.server_task.abort();
    tokio::fs::remove_dir_all(&self.workspace).await.unwrap();
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
    qr: Option<QrCodeFormat>,
    auth: Option<(String, String)>,
  ) -> DummyFile {
    let file_name: String = format!("{}.txt", Faker.fake::<String>());
    let content_type = "text/plain";
    let content = Faker.fake::<String>().as_bytes().to_vec();
    let query = UploadQueryParam {
      max_download: max,
      code_length: len,
      expire_secs: exp,
      allow_manual_deletion: del,
      qr_code_format: qr,
    };
    let (_, resp) = self
      .client
      .upload(
        file_name.clone(),
        content_type,
        content.clone(),
        &query,
        auth,
      )
      .await
      .unwrap();
    let resp = unwrap!(resp);
    let url_path = FileUrlPath::from_url(&resp.url).unwrap();
    DummyFile {
      content,
      content_type: content_type.to_string(),
      file_name,
      url_path,
    }
  }
}

#[derive(Clone)]
pub struct DummyFile {
  pub content: Vec<u8>,
  pub content_type: String,
  pub file_name: String,
  pub url_path: FileUrlPath,
}
