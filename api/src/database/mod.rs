use std::collections::HashMap;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use common::error::ApiResult;
use common::model::response::MetaDataFileResponse;
use tokio::sync::RwLock;

// code/file_name.ext
pub type PathFile = String;

#[derive(Default, Clone)]
pub struct DataBase {
  inner: Arc<RwLock<HashMap<PathFile, MetaData>>>,
}

impl DataBase {
  pub async fn fetch(&self, path: &PathFile) -> Option<MetaDataFile> {
    let guard = self.inner.read().await;
    guard.get(path).map(MetaDataFile::from)
  }

  pub async fn fetch_count(&self, path: &PathFile) -> Option<MetaDataFile> {
    let guard = self.inner.read().await;
    guard.get(path).map(|m| {
      let downloads = m
        .downloads
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
      MetaDataFile {
        create_at: m.create_at,
        expire_time: m.expire_time,
        password: m.password.clone(),
        is_deleteable: m.is_deleteable,
        max_download: m.max_download,
        downloads,
      }
    })
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
    guard.remove(path).map(MetaDataFile::from)
  }
}

#[derive(Clone)]
pub struct MetaDataFile {
  pub create_at: DateTime<Utc>,
  pub expire_time: DateTime<Utc>,
  pub password: Option<String>,
  pub is_deleteable: bool,
  pub max_download: Option<u32>,
  pub downloads: u32,
}

impl From<&MetaData> for MetaDataFile {
  fn from(value: &MetaData) -> Self {
    MetaDataFile {
      create_at: value.create_at,
      expire_time: value.expire_time,
      password: value.password.clone(),
      is_deleteable: value.is_deleteable,
      max_download: value.max_download,
      downloads: value.downloads.load(std::sync::atomic::Ordering::SeqCst),
    }
  }
}

impl From<MetaData> for MetaDataFile {
  fn from(value: MetaData) -> Self {
    MetaDataFile {
      create_at: value.create_at,
      expire_time: value.expire_time,
      password: value.password,
      is_deleteable: value.is_deleteable,
      max_download: value.max_download,
      downloads: value.downloads.load(std::sync::atomic::Ordering::SeqCst),
    }
  }
}

impl From<MetaDataFile> for MetaData {
  fn from(value: MetaDataFile) -> Self {
    MetaData {
      create_at: value.create_at,
      expire_time: value.expire_time,
      password: value.password,
      is_deleteable: value.is_deleteable,
      max_download: value.max_download,
      downloads: AtomicU32::new(value.downloads),
    }
  }
}

struct MetaData {
  pub create_at: DateTime<Utc>,
  pub expire_time: DateTime<Utc>,
  pub is_deleteable: bool,
  pub password: Option<String>,
  pub max_download: Option<u32>,
  pub downloads: AtomicU32,
}

impl From<&MetaDataFile> for MetaDataFileResponse {
  fn from(value: &MetaDataFile) -> Self {
    MetaDataFileResponse {
      create_at: value.create_at,
      expire_time: value.expire_time,
      is_deleteable: value.is_deleteable,
      max_download: value.max_download,
      downloads: value.downloads,
    }
  }
}
