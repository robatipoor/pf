use std::sync::atomic::AtomicU32;
use std::sync::Arc;
use std::time::Duration;
use std::{collections::HashMap, ops::Deref};

use chrono::{DateTime, Utc};
use common::error::ApiResult;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

// code/file_name.ext
pub type PathFile = String;

type DB = Arc<RwLock<HashMap<PathFile, (JoinHandle<()>, MetaDataFile)>>>;

#[derive(Default, Clone)]
pub struct DataBase {
  inner: DB,
}

impl DataBase {
  pub async fn find(&self, path: &PathFile) -> Option<MetaDataFile> {
    let guard = self.inner.read().await;
    guard.get(path).map(|(_, m)| m.clone())
  }
  pub async fn exist(&self, path: &PathFile) -> bool {
    let guard = self.inner.read().await;
    guard.get(path).is_some()
  }
  pub async fn store(&self, path: PathFile, meta: MetaDataFile) -> ApiResult {
    let mut guard = self.inner.write().await;
    let db = self.clone();
    let path_clone = path.clone();
    let f = tokio::spawn(async move {
      tokio::time::sleep(Duration::from_secs(10)).await;
      let mut guard = db.inner.write().await;
      guard.remove(&path_clone);
    });
    guard.insert(path, (f, meta));
    Ok(())
  }
  pub async fn delete(&self, path: &PathFile) -> Option<MetaDataFile> {
    let mut guard = self.inner.write().await;
    let data = guard.remove(path);
    if let Some((jh, data)) = data {
      jh.abort();
      Some(data)
    } else {
      None
    }
  }
}

#[derive(Clone)]
pub struct MetaDataFile {
  pub is_deleteable: bool,
  pub max_download: Option<u16>,
}
