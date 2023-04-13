use std::collections::{BTreeSet, HashMap};
use std::ops::Sub;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, DurationRound, Utc};
use common::error::ApiResult;
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
    while let Some((date, path)) = expires.iter().next() {
      if date < &now {
        db.remove(path);
        expires.remove(&(*date, path.to_string()));
      } else {
        // return Ok(Some(date.signed_duration_since(now)));
      }
    }
    Ok(None)
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

#[cfg(test)]
mod tests {
  use std::collections::{BTreeMap, BTreeSet};

  use chrono::{DateTime, Utc};
  use fake::{Fake, Faker};

  #[test]
  fn test_map() {
    let f1: DateTime<Utc> = Faker.fake();
    let f2: DateTime<Utc> = Faker.fake();
    let f3: DateTime<Utc> = Faker.fake();
    let f4: DateTime<Utc> = Faker.fake();
    let f5: DateTime<Utc> = Faker.fake();
    let mut b: BTreeSet<(DateTime<Utc>, u32)> = BTreeSet::new();
    b.insert((f1, 1));
    b.insert((f2, 2));
    b.insert((f3, 3));
    b.insert((f4, 4));
    b.insert((f5, 5));
    for k in b.iter() {
      println!("{k:?}");
    }
  }
}