use std::path::Path;
use std::path::PathBuf;
use std::{collections::HashMap, hash::Hash};

use crate::error::ApiResult;
use crate::server::worker::GarbageCollectorTask;
use crate::server::ApiState;
use crate::{configure::CONFIG, util::tracing::INIT_SUBSCRIBER};
use once_cell::sync::Lazy;
use test_context::AsyncTestContext;

pub struct StateTestContext {
  pub state: ApiState,
  gc_task: tokio::task::JoinHandle<ApiResult>,
}

#[async_trait::async_trait]
impl AsyncTestContext for StateTestContext {
  async fn setup() -> Self {
    Lazy::force(&INIT_SUBSCRIBER);
    let workspace = Path::new("test-dump").join(PathBuf::from(cuid2::create_id()));
    let db_path = Path::new("test-dump").join(PathBuf::from(cuid2::create_id()));
    tokio::fs::create_dir_all(&workspace).await.unwrap();
    let mut config = CONFIG.clone();
    config.fs.base_dir = workspace;
    config.db.path_dir = db_path;
    let state = ApiState::new(config).unwrap();
    let gc_task = tokio::task::spawn(GarbageCollectorTask::new(state.clone()).run());
    Self { state, gc_task }
  }

  async fn teardown(self) {
    self.gc_task.abort();
    tokio::fs::remove_dir_all(&self.state.config.db.path_dir)
      .await
      .unwrap();
    tokio::fs::remove_dir_all(&self.state.config.fs.base_dir)
      .await
      .unwrap();
  }
}

pub fn eq<T>(a: &[T], b: &[T]) -> bool
where
  T: Eq + Hash,
{
  fn count<T>(items: &[T]) -> HashMap<&T, usize>
  where
    T: Eq + Hash,
  {
    let mut cnt = HashMap::new();
    for i in items {
      *cnt.entry(i).or_insert(0) += 1
    }
    cnt
  }
  count(a) == count(b)
}

pub fn vecs_match<T: PartialEq>(a: &[T], b: &[T]) -> bool {
  a.len() == b.len() && !a.iter().zip(b.iter()).any(|(a, b)| *a != *b)
}
