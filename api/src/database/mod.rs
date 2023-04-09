use std::sync::atomic::AtomicU32;
use std::sync::Arc;
use std::time::Duration;
use std::{collections::HashMap, ops::Deref};

use chrono::{DateTime, Utc};
use common::error::ApiResult;
use tokio::sync::RwLock;

// code/filename.ext
pub type PathFile = String;

#[derive(Default)]
pub struct DataBase {
  inner: Arc<RwLock<HashMap<PathFile, MetaDataFile>>>,
  seq: Sequential,
}

impl DataBase {
  pub async fn find_by_path_file(&self, path: &PathFile) -> Option<MetaDataFile> {
    let guard = self.inner.read().await;
    guard.get(path).map(Clone::clone)
  }
  pub async fn exist_by_path_file(&self, path: &PathFile) -> bool {
    let guard = self.inner.read().await;
    guard.get(path).is_some()
  }
  pub async fn save(&self, path: PathFile, meta: MetaDataFile) -> ApiResult {
    let mut guard = self.inner.write().await;
    guard.insert(path, meta);
    Ok(())
  }
  pub async fn remove_by_path_file(&self, path: &PathFile) -> Option<MetaDataFile> {
    let mut guard = self.inner.write().await;
    guard.remove(path)
  }
}

#[derive(Default)]
pub struct Sequential {
  inner: AtomicU32,
}

impl Sequential {
  pub fn get_id(&self) -> Option<u32> {
    Some(self.inner.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
  }
}

impl Deref for DataBase {
  type Target = Sequential;

  fn deref(&self) -> &Self::Target {
    &self.seq
  }
}

#[derive(Clone)]
pub struct MetaDataFile {
  pub id: u32,
  pub expire_time: Duration,
  pub is_deleteable: bool,
  pub max_download: Option<u16>,
  pub create_at: DateTime<Utc>,
}
