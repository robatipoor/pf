use std::collections::HashMap;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;

use common::error::ApiResult;
use tokio::sync::RwLock;
use tokio::time::Instant;

// code/file_name.ext
pub type PathFile = String;

#[derive(Default, Clone)]
pub struct DataBase {
  inner: Arc<RwLock<HashMap<PathFile, MetaData>>>,
}

impl DataBase {
  pub async fn fetch_count(&self, path: &PathFile) -> Option<MetaDataFile> {
    let guard = self.inner.read().await;
    guard
      .get(path)
      .filter(|m| m.expire_time > Instant::now())
      .filter(|m| {
        let count = m
          .downloads
          .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        match m.max_download {
          Some(max) if max >= count => true,
          None => true,
          _ => false,
        }
      })
      .map(MetaDataFile::from)
  }

  pub async fn fetch_any(&self, path: &PathFile) -> Option<MetaDataFile> {
    let guard = self.inner.read().await;
    guard.get(path).map(MetaDataFile::from)
  }

  pub async fn exist(&self, path: &PathFile) -> bool {
    let guard = self.inner.read().await;
    guard.get(path).is_some()
  }
  pub async fn store(&self, path: PathFile, meta: MetaDataFile) -> ApiResult {
    let mut guard = self.inner.write().await;
    guard.insert(path, meta.into());
    Ok(())
  }
  pub async fn delete(&self, path: &PathFile) -> Option<MetaDataFile> {
    let mut guard = self.inner.write().await;
    match guard.get(path) {
      Some(data) if data.is_deleteable => guard.remove(path).map(MetaDataFile::from),
      _ => None,
    }
  }
}

#[derive(Clone)]
pub struct MetaDataFile {
  pub create_at: Instant,
  pub expire_time: Instant,
  pub is_deleteable: bool,
  pub max_download: Option<u32>,
  pub downloads: u32,
}

impl From<&MetaData> for MetaDataFile {
  fn from(value: &MetaData) -> Self {
    MetaDataFile {
      create_at: value.create_at,
      expire_time: value.expire_time,
      is_deleteable: value.is_deleteable,
      max_download: value.max_download,
      downloads: value.downloads.load(std::sync::atomic::Ordering::SeqCst),
    }
  }
}

impl From<MetaData> for MetaDataFile {
  fn from(value: MetaData) -> Self {
    MetaDataFile::from(&value)
  }
}

impl From<&MetaDataFile> for MetaData {
  fn from(value: &MetaDataFile) -> Self {
    MetaData {
      create_at: value.create_at,
      expire_time: value.expire_time,
      is_deleteable: value.is_deleteable,
      max_download: value.max_download,
      downloads: AtomicU32::new(value.downloads),
    }
  }
}

impl From<MetaDataFile> for MetaData {
  fn from(value: MetaDataFile) -> Self {
    MetaData::from(&value)
  }
}

struct MetaData {
  pub create_at: Instant,
  pub expire_time: Instant,
  pub is_deleteable: bool,
  pub max_download: Option<u32>,
  pub downloads: AtomicU32,
}
