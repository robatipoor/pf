use std::path::Path;
use std::path::PathBuf;
use std::{collections::HashMap, hash::Hash};

use crate::server::ApiState;
use crate::{configure::CONFIG, util::tracing::INIT_SUBSCRIBER};
use once_cell::sync::Lazy;
use test_context::AsyncTestContext;

#[macro_export]
macro_rules! assert_ok {
  ($result:expr) => {
    assert!(
      matches!($result, sdk::result::ApiResponseResult::Ok(_)),
      "match failed: {:?}",
      $result,
    )
  };
}

#[macro_export]
macro_rules! assert_err {
    ($result:expr $(, $closure:expr )?) => {
        assert!(
          matches!($result,sdk::result::ApiResponseResult::Err(ref _e) $( if $closure(_e) )?),
          "match failed: {:?}",$result,
        )
    };
}

#[macro_export]
macro_rules! unwrap {
  ($result:expr) => {
    match $result {
      sdk::result::ApiResponseResult::Ok(resp) => resp,
      sdk::result::ApiResponseResult::Err(e) => {
        panic!("called `util::unwrap!()` on an `Err` value {e:?}")
      }
    }
  };
}

pub struct StateTestContext {
  pub state: ApiState,
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
    config.db.path = db_path;
    let state = ApiState::new(config).unwrap();
    crate::server::worker::spawn(axum::extract::State(state.clone()));
    Self { state }
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

pub fn vecs_match<T: PartialEq>(a: &Vec<T>, b: &Vec<T>) -> bool {
  a.len() == b.len() && !a.iter().zip(b.iter()).any(|(a, b)| *a != *b)
}
