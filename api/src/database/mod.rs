use std::sync::Arc;
use std::time::Duration;
use std::{collections::BTreeSet, path::PathBuf};

use crate::util::secret::SecretHash;
use crate::{
  configure::DatabaseConfig,
  error::{ApiError, ApiResult},
};
use chrono::{DateTime, Utc};
use sdk::model::response::MetaDataFileResponse;
use serde::{Deserialize, Serialize};
use sled::IVec;
use std::sync::RwLock;
use tokio::sync::Notify;

pub type Expires = Arc<RwLock<BTreeSet<(DateTime<Utc>, FilePath)>>>;

#[derive(Clone)]
pub struct Database {
  inner: sled::Db,
  expires: Expires,
  notify: Arc<Notify>,
}

impl Database {
  pub fn new(config: &DatabaseConfig) -> ApiResult<Self> {
    let db = sled::open(&config.path)?;
    let mut expires = BTreeSet::new();
    for kv in db.iter() {
      let (key, val) = kv?;
      let file_path = FilePath::try_from(&key)?;
      let expire_time = MetaDataFile::try_from(val)?.expiration_date;
      expires.insert((expire_time, file_path));
    }
    Ok(Self {
      inner: db,
      expires: Arc::new(RwLock::new(expires)),
      notify: Default::default(),
    })
  }

  pub fn fetch(&self, file_path: &FilePath) -> ApiResult<Option<MetaDataFile>> {
    self
      .inner
      .get(IVec::try_from(file_path)?)?
      .map(MetaDataFile::try_from)
      .transpose()
  }

  pub async fn fetch_count(&self, file_path: &FilePath) -> ApiResult<Option<MetaDataFile>> {
    let mut output = None;
    self
      .inner
      .fetch_and_update(IVec::try_from(file_path)?, |value| {
        value
          .map(|val| {
            MetaDataFile::try_from(val)
              .map(|mut meta| {
                meta.count_downloads += 1;
                let val = IVec::try_from(&meta);
                output = Some(meta);
                val
                  .map_err(|err| {
                    tracing::error!("Covnert MetaDataFile to IVec unsuccessfully: {err}");
                    err
                  })
                  .ok()
              })
              .map_err(|err| {
                tracing::error!("Covnert IVec to MetaDataFile unsuccessfully: {err}");
                err
              })
              .ok()
          })
          .flatten()
          .flatten()
      })?;
    Ok(output)
  }

  pub fn exist(&self, path: &FilePath) -> ApiResult<bool> {
    Ok(self.inner.contains_key(IVec::try_from(path)?)?)
  }

  pub async fn store(&self, path: FilePath, meta: MetaDataFile) -> ApiResult {
    let expire_time = meta.expiration_date;
    let meta = IVec::try_from(&meta)?;
    let key = IVec::try_from(&path)?;
    let result = self
      .inner
      .compare_and_swap(&key, Option::<IVec>::None, Some(meta))?;
    match result {
      Ok(_) => {
        let expire = (expire_time, path);
        match self.expires.write() {
          Ok(mut guard) => {
            let is_gc_notify = guard
              .iter()
              .next()
              .filter(|(exp, _)| *exp < Utc::now())
              .is_some();
            guard.insert(expire.clone());
            drop(guard);
            if is_gc_notify {
              self.notify_gc();
            }
          }
          Err(err) => {
            self.inner.remove(&key)?;
            return Err(ApiError::LockError(err.to_string()));
          }
        }
      }
      Err(err) if err.current.is_some() => {
        return Err(ApiError::ResourceExistsError("File path exists".to_string()));
      }
      Err(err) => {
        tracing::error!("Compare and swap error: {err}");
        return Err(ApiError::DatabaseError(sled::Error::ReportableBug(
          "Storing the meta data file in the database faild.".to_string(),
        )));
      }
    };
    Ok(())
  }

  pub async fn delete(&self, path: FilePath) -> ApiResult<Option<MetaDataFile>> {
    let key = IVec::try_from(&path)?;
    if let Some(meta) = self
      .inner
      .remove(&key)?
      .map(MetaDataFile::try_from)
      .transpose()?
    {
      match self.expires.write() {
        Ok(mut guard) => {
          guard.remove(&(meta.expiration_date, path));
        }
        Err(err) => {
          tracing::error!("Get expires lock unsuccessfully: {err}");
        }
      }
      Ok(Some(meta))
    } else {
      Ok(None)
    }
  }

  pub async fn purge(&self) -> ApiResult<Option<Duration>> {
    let now = Utc::now();
    match self.expires.write() {
      Ok(mut guard) => {
        let expires = &mut *guard;
        while let Some((expire_date, path)) = expires.iter().next().cloned() {
          if expire_date < now {
            self.inner.remove(&IVec::try_from(&path)?)?;
            expires.remove(&(expire_date, path));
          } else {
            return Ok(Some((expire_date - now).to_std()?));
          }
        }
      }
      Err(err) => {
        tracing::error!("Get expires lock unsuccessfully: {err}");
        return Err(ApiError::LockError(err.to_string()));
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

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, fake::Dummy)]
pub struct FilePath {
  pub code: String,
  pub file_name: String,
}

impl FilePath {
  pub fn url(&self, domain: &str) -> String {
    format!("{domain}/{}/{}", self.code, self.file_name)
  }
  pub fn url_path(&self) -> String {
    format!("{}/{}", self.code, self.file_name)
  }
  pub fn fs_path(&self, base_dir: &PathBuf) -> PathBuf {
    base_dir.join(format!("{}/{}", self.code, self.file_name))
  }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MetaDataFile {
  pub created_at: DateTime<Utc>,
  pub expiration_date: DateTime<Utc>,
  pub secret: Option<SecretHash>,
  pub delete_manually: bool,
  pub max_download: Option<u32>,
  pub count_downloads: u32,
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

impl TryFrom<&IVec> for FilePath {
  type Error = ApiError;

  fn try_from(value: &IVec) -> ApiResult<Self> {
    let value = bincode::deserialize::<Self>(value)?;
    Ok(value)
  }
}

impl TryFrom<IVec> for FilePath {
  type Error = ApiError;

  fn try_from(value: IVec) -> ApiResult<Self> {
    Self::try_from(&value)
  }
}

impl TryFrom<&FilePath> for IVec {
  type Error = ApiError;
  fn try_from(value: &FilePath) -> ApiResult<IVec> {
    let value = bincode::serialize(value)?;
    Ok(IVec::from(value))
  }
}

impl TryFrom<FilePath> for IVec {
  type Error = ApiError;
  fn try_from(value: FilePath) -> ApiResult<IVec> {
    Self::try_from(&value)
  }
}

impl From<&MetaDataFile> for MetaDataFileResponse {
  fn from(value: &MetaDataFile) -> Self {
    MetaDataFileResponse {
      created_at: value.created_at,
      expiration_date: value.expiration_date,
      delete_manually: value.delete_manually,
      max_download: value.max_download,
      count_downloads: value.count_downloads,
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
  async fn test_store_file_and_fetch(ctx: &mut StateTestContext) {
    let path: FilePath = Faker.fake();
    let meta = MetaDataFile {
      created_at: Utc::now(),
      expiration_date: Utc::now() + chrono::Duration::seconds(10),
      secret: None,
      delete_manually: true,
      max_download: None,
      count_downloads: 1,
    };
    ctx
      .state
      .db
      .store(path.clone(), meta.clone())
      .await
      .unwrap();
    let result = ctx.state.db.fetch(&path).unwrap().unwrap();
    assert_eq!(result.created_at, meta.created_at);
    assert_eq!(result.expiration_date, meta.expiration_date);
    assert_eq!(result.secret, meta.secret);
    assert_eq!(result.max_download, meta.max_download);
    assert_eq!(result.count_downloads, meta.count_downloads);
  }

  #[test_context(StateTestContext)]
  #[tokio::test]
  async fn test_store_file_and_fetch_count(ctx: &mut StateTestContext) {
    let path: FilePath = Faker.fake();
    let meta = MetaDataFile {
      created_at: Utc::now(),
      expiration_date: Utc::now() + chrono::Duration::seconds(10),
      secret: None,
      delete_manually: true,
      max_download: None,
      count_downloads: 0,
    };
    ctx
      .state
      .db
      .store(path.clone(), meta.clone())
      .await
      .unwrap();
    let result = ctx.state.db.fetch_count(&path).await.unwrap().unwrap();
    assert_eq!(result.created_at, meta.created_at);
    assert_eq!(result.expiration_date, meta.expiration_date);
    assert_eq!(result.secret, meta.secret);
    assert_eq!(result.max_download, meta.max_download);
    assert_eq!(result.count_downloads, meta.count_downloads);
  }

  #[test_context(StateTestContext)]
  #[tokio::test]
  async fn test_store_file_and_double_fetch_count(ctx: &mut StateTestContext) {
    let path: FilePath = Faker.fake();
    let meta = MetaDataFile {
      created_at: Utc::now(),
      expiration_date: Utc::now() + chrono::Duration::seconds(10),
      secret: None,
      delete_manually: true,
      max_download: None,
      count_downloads: 0,
    };
    ctx
      .state
      .db
      .store(path.clone(), meta.clone())
      .await
      .unwrap();
    ctx.state.db.fetch_count(&path).await.unwrap().unwrap();
    let result = ctx.state.db.fetch_count(&path).await.unwrap().unwrap();
    assert_eq!(result.created_at, meta.created_at);
    assert_eq!(result.expiration_date, meta.expiration_date);
    assert_eq!(result.secret, meta.secret);
    assert_eq!(result.max_download, meta.max_download);
    assert_eq!(result.count_downloads, meta.count_downloads + 1);
  }

  #[test_context(StateTestContext)]
  #[tokio::test]
  async fn test_store_file_and_check_it_existence(ctx: &mut StateTestContext) {
    let path: FilePath = Faker.fake();
    let meta = MetaDataFile {
      created_at: Utc::now(),
      expiration_date: Utc::now() + chrono::Duration::seconds(10),
      secret: None,
      delete_manually: true,
      max_download: None,
      count_downloads: 0,
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
  async fn test_store_file_and_purge_it(ctx: &mut StateTestContext) {
    let path: FilePath = Faker.fake();
    let meta = MetaDataFile {
      created_at: Utc::now(),
      expiration_date: Utc::now(),
      secret: None,
      delete_manually: true,
      max_download: None,
      count_downloads: 0,
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
  async fn test_store_file_and_successfully_delete_it(ctx: &mut StateTestContext) {
    let path: FilePath = Faker.fake();
    let meta = MetaDataFile {
      created_at: Utc::now(),
      expiration_date: Utc::now() + chrono::Duration::seconds(10),
      secret: None,
      delete_manually: true,
      max_download: None,
      count_downloads: 0,
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
  async fn test_delete_file_that_does_not_exist(ctx: &mut StateTestContext) {
    let path: FilePath = Faker.fake();
    let result = ctx.state.db.delete(path).await.unwrap();
    assert!(result.is_none())
  }
}
