use std::collections::{BTreeSet, HashMap};
use std::sync::atomic::AtomicU32;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use common::error::{ApiError, ApiResult};
use common::model::response::MetaDataFileResponse;
use tokio::sync::RwLock;

// code/file_name.ext
pub type PathFile = String;

pub type Expires = Arc<RwLock<BTreeSet<(DateTime<Utc>, PathFile)>>>;

type Db = Arc<RwLock<HashMap<PathFile, MetaData>>>;

#[derive(Default, Clone)]
pub struct DataBase {
  inner: Db,
  expires: Expires,
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
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        + 1;
      MetaDataFile {
        create_at: m.create_at,
        expire_time: m.expire_time,
        auth: m.auth.clone(),
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
    self
      .expires
      .write()
      .await
      .insert((meta.expire_time, path.clone()));
    self.inner.write().await.insert(path, meta.into());
    // trigger gc
    Ok(())
  }
  pub async fn delete(&self, path: PathFile) -> Option<MetaDataFile> {
    if let Some(meta) = self
      .inner
      .write()
      .await
      .remove(&path)
      .map(MetaDataFile::from)
    {
      self.expires.write().await.remove(&(meta.expire_time, path));
      Some(meta)
    } else {
      None
    }
  }

  pub async fn purge(&self) -> ApiResult<Option<Duration>> {
    let mut expires = self.expires.write().await;
    let mut db = self.inner.write().await;
    let expires = &mut *expires;
    let db = &mut *db;
    let now = Utc::now();
    while let Some((date, path)) = expires.iter().next().cloned() {
      if date < now {
        db.remove(&path);
        expires.remove(&(date, path));
      } else {
        return Ok(Some((date - now).to_std().map_err(|e| {
          ApiError::Unknown(anyhow::anyhow!("convert duration failed: {e}"))
        })?));
      }
    }
    Ok(None)
  }

  pub async fn first_expire(&self) -> Option<DateTime<Utc>> {
    let expires = self.expires.read().await;
    expires.iter().next().map(|(d, _)| *d)
  }
}

#[derive(Clone)]
pub struct MetaDataFile {
  pub create_at: DateTime<Utc>,
  pub expire_time: DateTime<Utc>,
  pub auth: Option<String>,
  pub is_deleteable: bool,
  pub max_download: Option<u32>,
  pub downloads: u32,
}

impl From<&MetaData> for MetaDataFile {
  fn from(value: &MetaData) -> Self {
    MetaDataFile {
      create_at: value.create_at,
      expire_time: value.expire_time,
      auth: value.auth.clone(),
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
      auth: value.auth,
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
      auth: value.auth,
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
  pub auth: Option<String>,
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

#[cfg(test)]
mod tests {
  use std::collections::BTreeSet;

  use chrono::{DateTime, Utc};
  use fake::{Fake, Faker};
}
