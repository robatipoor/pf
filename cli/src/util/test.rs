use fake::{Fake, Faker};
use test_context::AsyncTestContext;

pub struct FileTestContext {
  pub temp_path: std::path::PathBuf,
}

#[async_trait::async_trait]
impl AsyncTestContext for FileTestContext {
  async fn setup() -> Self {
    let temp_path = std::path::PathBuf::from(&format!("test_dir_{}", Faker.fake::<String>()));
    tokio::fs::create_dir_all(&temp_path).await.unwrap();
    Self { temp_path }
  }

  async fn teardown(self) {
    tokio::fs::remove_dir_all(self.temp_path).await.unwrap();
  }
}
