use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Duration;

use crate::{
  config::DatabaseConfig,
  error::{ApiError, ApiResult},
};
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use sdk::model::response::MetaDataFileResponse;
use serde::{Deserialize, Serialize};
use sled::IVec;
use tokio::sync::{Notify, RwLock};
use tracing::error;

/// code/file_name.ext
pub type PathFile = String;

pub type Expires = Arc<RwLock<BTreeSet<(DateTime<Utc>, PathFile)>>>;

#[derive(Clone)]
pub struct Database {
  inner: sled::Db,
  expires: Expires,
  notify: Arc<Notify>,
}

impl Database {
  pub fn new(config: &DatabaseConfig) -> ApiResult<Self> {
    let inner = sled::open(&config.path)?;
    let mut tree = BTreeSet::new();
    for v in inner.iter() {
      let (key, val) = v?;
      let path = std::str::from_utf8(&key)?.to_string();
      let expire_time = MetaDataFile::try_from(val)?.expire_time;
      tree.insert((expire_time, path));
    }
    Ok(Self {
      inner,
      expires: Arc::new(RwLock::new(tree)),
      notify: Default::default(),
    })
  }

  pub fn fetch(&self, path: &PathFile) -> ApiResult<Option<MetaDataFile>> {
    let result = self.inner.get(path)?;
    result.map(MetaDataFile::try_from).transpose()
  }

  pub async fn fetch_count(&self, path: &PathFile) -> ApiResult<Option<MetaDataFile>> {
    let value = self.inner.fetch_and_update(path, |meta| {
      if let Some(value) = meta {
        let mut meta = match MetaDataFile::try_from(value) {
          Ok(m) => m,
          Err(e) => {
            error!("convert bytes to MetaDataFile failed: {e}");
            return None;
          }
        };
        meta.downloads += 1;
        let value: IVec = meta.try_into().unwrap();
        Some(value)
      } else {
        None
      }
    })?;
    if let Some(value) = value {
      Ok(Some(MetaDataFile::try_from(value)?))
    } else {
      Ok(None)
    }
  }

  pub fn exist(&self, path: &PathFile) -> ApiResult<bool> {
    let result = self.inner.contains_key(path)?;
    Ok(result)
  }

  pub async fn store(&self, path: PathFile, meta: MetaDataFile) -> ApiResult {
    let expire_time = meta.expire_time;
    let meta: IVec = meta.try_into()?;
    let mut guard = self.expires.write().await;
    let first = guard.iter().next().map(|(d, _)| *d);
    let expire = (expire_time, path.clone());
    guard.insert(expire.clone());
    let result = self
      .inner
      .compare_and_swap(&path, Option::<IVec>::None, Some(meta));
    match result {
      Ok(Ok(_)) => (),
      Err(e) => {
        guard.remove(&expire);
        return Err(e.into());
      }
      Ok(Err(e)) if e.current.is_some() => {
        guard.remove(&expire);
        return Err(ApiError::ResourceExists("path exists".to_string()));
      }
      _ => {
        guard.remove(&expire);
        return Err(ApiError::Unknown(anyhow!("unexpected error".to_string())));
      }
    }
    drop(guard);
    let notify = match first {
      Some(e) if e > expire_time => true,
      None => true,
      _ => false,
    };
    if notify {
      self.notify_gc();
    }
    Ok(())
  }

  pub async fn delete(&self, path: PathFile) -> ApiResult<Option<MetaDataFile>> {
    if let Some(meta) = self
      .inner
      .remove(&path)?
      .map(MetaDataFile::try_from)
      .transpose()?
    {
      self.expires.write().await.remove(&(meta.expire_time, path));
      Ok(Some(meta))
    } else {
      Ok(None)
    }
  }

  pub async fn purge(&self) -> ApiResult<Option<Duration>> {
    let mut expires = self.expires.write().await;
    let expires = &mut *expires;
    let now = Utc::now();
    while let Some((date, path)) = expires.iter().next().cloned() {
      if date < now {
        self.inner.remove(&path)?;
        expires.remove(&(date, path));
      } else {
        return Ok(Some((date - now).to_std().map_err(|e| {
          ApiError::Unknown(anyhow::anyhow!("convert duration failed: {e}"))
        })?));
      }
    }
    Ok(None)
  }

  fn notify_gc(&self) {
    self.notify.notify_one()
  }

  pub async fn flush(&self) -> ApiResult {
    self.inner.flush_async().await?;
    Ok(())
  }

  pub async fn waiting_for_notify(&self) {
    self.notify.notified().await
  }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MetaDataFile {
  pub create_at: DateTime<Utc>,
  pub expire_time: DateTime<Utc>,
  pub auth: Option<String>,
  pub is_deleteable: bool,
  pub max_download: Option<u32>,
  pub downloads: u32,
}

impl TryFrom<&[u8]> for MetaDataFile {
  type Error = ApiError;

  fn try_from(value: &[u8]) -> ApiResult<Self> {
    let value = bincode::deserialize::<Self>(value)?;
    Ok(value)
  }
}

impl TryFrom<&IVec> for MetaDataFile {
  type Error = ApiError;

  fn try_from(value: &IVec) -> ApiResult<Self> {
    let value = bincode::deserialize::<Self>(value)?;
    Ok(value)
  }
}

impl TryFrom<IVec> for MetaDataFile {
  type Error = ApiError;

  fn try_from(value: IVec) -> ApiResult<Self> {
    Self::try_from(&value)
  }
}

impl TryFrom<&MetaDataFile> for IVec {
  type Error = ApiError;
  fn try_from(value: &MetaDataFile) -> ApiResult<IVec> {
    let value = bincode::serialize(value)?;
    Ok(IVec::from(value))
  }
}

impl TryFrom<MetaDataFile> for IVec {
  type Error = ApiError;
  fn try_from(value: MetaDataFile) -> ApiResult<IVec> {
    Self::try_from(&value)
  }
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
  use super::*;
  use crate::util::test::StateTestContext;
  use fake::{Fake, Faker};
  use test_context::test_context;

  #[test_context(StateTestContext)]
  #[tokio::test]
  async fn store_file_and_fetch_test(ctx: &mut StateTestContext) {
    let path: String = Faker.fake();
    let meta = MetaDataFile {
      create_at: Utc::now(),
      expire_time: Utc::now() + chrono::Duration::seconds(10),
      auth: None,
      is_deleteable: true,
      max_download: None,
      downloads: 1,
    };
    ctx
      .state
      .db
      .store(path.clone(), meta.clone())
      .await
      .unwrap();
    let result = ctx.state.db.fetch(&path).unwrap().unwrap();
    assert_eq!(result.create_at, meta.create_at);
    assert_eq!(result.expire_time, meta.expire_time);
    assert_eq!(result.auth, meta.auth);
    assert_eq!(result.max_download, meta.max_download);
    assert_eq!(result.downloads, meta.downloads);
  }

  #[test_context(StateTestContext)]
  #[tokio::test]
  async fn store_file_and_fetch_count_test(ctx: &mut StateTestContext) {
    let path: String = Faker.fake();
    let meta = MetaDataFile {
      create_at: Utc::now(),
      expire_time: Utc::now() + chrono::Duration::seconds(10),
      auth: None,
      is_deleteable: true,
      max_download: None,
      downloads: 0,
    };
    ctx
      .state
      .db
      .store(path.clone(), meta.clone())
      .await
      .unwrap();
    let result = ctx.state.db.fetch_count(&path).await.unwrap().unwrap();
    assert_eq!(result.create_at, meta.create_at);
    assert_eq!(result.expire_time, meta.expire_time);
    assert_eq!(result.auth, meta.auth);
    assert_eq!(result.max_download, meta.max_download);
    assert_eq!(result.downloads, meta.downloads);
  }

  #[test_context(StateTestContext)]
  #[tokio::test]
  async fn store_file_and_double_fetch_count_test(ctx: &mut StateTestContext) {
    let path: String = Faker.fake();
    let meta = MetaDataFile {
      create_at: Utc::now(),
      expire_time: Utc::now() + chrono::Duration::seconds(10),
      auth: None,
      is_deleteable: true,
      max_download: None,
      downloads: 0,
    };
    ctx
      .state
      .db
      .store(path.clone(), meta.clone())
      .await
      .unwrap();
    ctx.state.db.fetch_count(&path).await.unwrap().unwrap();
    let result = ctx.state.db.fetch_count(&path).await.unwrap().unwrap();
    assert_eq!(result.create_at, meta.create_at);
    assert_eq!(result.expire_time, meta.expire_time);
    assert_eq!(result.auth, meta.auth);
    assert_eq!(result.max_download, meta.max_download);
    assert_eq!(result.downloads, meta.downloads + 1);
  }

  #[test_context(StateTestContext)]
  #[tokio::test]
  async fn store_file_and_check_it_existence_test(ctx: &mut StateTestContext) {
    let path: String = Faker.fake();
    let meta = MetaDataFile {
      create_at: Utc::now(),
      expire_time: Utc::now() + chrono::Duration::seconds(10),
      auth: None,
      is_deleteable: true,
      max_download: None,
      downloads: 0,
    };
    ctx
      .state
      .db
      .store(path.clone(), meta.clone())
      .await
      .unwrap();
    let result = ctx.state.db.exist(&path).unwrap();
    assert!(result);
  }

  #[test_context(StateTestContext)]
  #[tokio::test]
  async fn store_file_and_purge_it_test(ctx: &mut StateTestContext) {
    let path: String = Faker.fake();
    let meta = MetaDataFile {
      create_at: Utc::now(),
      expire_time: Utc::now(),
      auth: None,
      is_deleteable: true,
      max_download: None,
      downloads: 0,
    };
    ctx
      .state
      .db
      .store(path.clone(), meta.clone())
      .await
      .unwrap();
    tokio::time::sleep(Duration::from_secs(1)).await;
    let result = ctx.state.db.exist(&path).unwrap();
    assert!(!result);
  }

  #[test_context(StateTestContext)]
  #[tokio::test]
  async fn store_file_and_successfully_delete_it_test(ctx: &mut StateTestContext) {
    let path: String = Faker.fake();
    let meta = MetaDataFile {
      create_at: Utc::now(),
      expire_time: Utc::now() + chrono::Duration::seconds(10),
      auth: None,
      is_deleteable: true,
      max_download: None,
      downloads: 0,
    };
    ctx
      .state
      .db
      .store(path.clone(), meta.clone())
      .await
      .unwrap();
    ctx.state.db.delete(path).await.unwrap().unwrap();
  }

  #[test_context(StateTestContext)]
  #[tokio::test]
  async fn delete_file_that_does_not_exist_test(ctx: &mut StateTestContext) {
    let path: String = Faker.fake();
    let result = ctx.state.db.delete(path).await.unwrap();
    assert!(result.is_none())
  }
}
