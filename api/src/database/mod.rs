use crate::{
  configure::DatabaseConfig,
  error::{result::ApiResult, ApiError},
};
use chrono::{DateTime, Utc};
use sled::IVec;
use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;
use tokio::sync::Notify;

use self::file_path::FilePath;
use self::meta_data_file::MetaDataFile;

pub mod file_path;
pub mod meta_data_file;

pub type Expires = Arc<RwLock<BTreeSet<(DateTime<Utc>, FilePath)>>>;

#[derive(Clone)]
pub struct Database {
  inner: sled::Db,
  expires: Expires,
  notify: Arc<Notify>,
}

impl Database {
  pub fn new(config: &DatabaseConfig) -> ApiResult<Self> {
    let db = sled::open(&config.path_dir)?;
    let expires = Self::load_expires(&db)?;
    Ok(Self {
      inner: db,
      expires: Arc::new(RwLock::new(expires)),
      notify: Default::default(),
    })
  }

  fn load_expires(db: &sled::Db) -> ApiResult<BTreeSet<(DateTime<Utc>, FilePath)>> {
    let mut expires = BTreeSet::new();
    for kv in db.iter() {
      let (key, val) = kv?;
      let file_path = FilePath::try_from(&key)?;
      let expire_time = MetaDataFile::try_from(val)?.expire_date_time;
      expires.insert((expire_time, file_path));
    }
    Ok(expires)
  }

  pub fn fetch(&self, file_path: &FilePath) -> ApiResult<Option<MetaDataFile>> {
    self
      .inner
      .get(IVec::try_from(file_path)?)?
      .map(MetaDataFile::try_from)
      .transpose()
  }

  pub fn update(&self, file_path: &FilePath, old: MetaDataFile, new: MetaDataFile) -> ApiResult {
    let old = IVec::try_from(old)?;
    let new = IVec::try_from(new)?;
    let file_path = IVec::try_from(file_path)?;
    let result = self
      .inner
      .compare_and_swap(&file_path, Some(old), Some(new))?;
    match result {
      Ok(_) => Ok(()),
      Err(err) if err.current.is_some() => {
        tracing::warn!("Compare and swap failed, Error: {err}");
        Err(ApiError::BadRequestError(
          "Updating the meta data file in the database failed.".to_string(),
        ))
      }
      Err(err) => {
        tracing::error!("Compare and swap failed, Error: {err}");
        Err(ApiError::DatabaseError(sled::Error::ReportableBug(
          format!("Updating the meta data file in the database failed, Error: {err}"),
        )))
      }
    }
  }

  pub fn exist(&self, path: &FilePath) -> ApiResult<bool> {
    Ok(self.inner.contains_key(IVec::try_from(path)?)?)
  }

  pub async fn store(&self, path: FilePath, meta: MetaDataFile) -> ApiResult {
    let expire_date_time = meta.expire_date_time;
    let meta = IVec::try_from(&meta)?;
    let key = IVec::try_from(&path)?;
    let result = self
      .inner
      .compare_and_swap(&key, Option::<IVec>::None, Some(meta))?;
    match result {
      Ok(_) => {
        let expire = (expire_date_time, path);
        match self.expires.write() {
          Ok(mut guard) => {
            let is_gc_notify = guard
              .iter()
              .next()
              .map_or(true, |(first_expire, _)| *first_expire > expire_date_time);
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
        return Err(ApiError::ResourceExistsError(
          "File path exists".to_string(),
        ));
      }
      Err(err) => {
        tracing::error!("Compare and swap error, Error: {err}");
        return Err(ApiError::DatabaseError(sled::Error::ReportableBug(
          "Storing the meta data file in the database failed.".to_string(),
        )));
      }
    };
    Ok(())
  }

  pub async fn delete(&self, path: FilePath) -> ApiResult<Option<MetaDataFile>> {
    let key = IVec::try_from(&path)?;
    let meta = self
      .inner
      .remove(&key)?
      .map(MetaDataFile::try_from)
      .transpose()?;
    if let Some(meta) = &meta {
      match self.expires.write() {
        Ok(mut guard) => {
          guard.remove(&(meta.expire_date_time, path));
        }
        Err(err) => {
          tracing::error!("Failed to acquire expires lock, Error: {err}");
        }
      }
    }
    Ok(meta)
  }

  pub async fn purge(&self) -> ApiResult<Option<Duration>> {
    let mut paths_should_delete = vec![];
    let mut wakeup_next_time = None;
    match self.expires.write() {
      Ok(mut guard) => {
        let expires = &mut *guard;
        while let Some((expire_date, path)) = expires.iter().next().cloned() {
          let now = Utc::now();
          if expire_date < now {
            expires.remove(&(expire_date, path.clone()));
            paths_should_delete.push(path);
          } else {
            wakeup_next_time = Some((expire_date - now).to_std()?);
            break;
          }
        }
      }
      Err(err) => {
        tracing::error!("Failed to acquire expires lock, Error: {err}");
        return Err(ApiError::LockError(err.to_string()));
      }
    }
    self.remove_file(paths_should_delete).await?;
    Ok(wakeup_next_time)
  }

  pub async fn remove_file(&self, paths: Vec<FilePath>) -> ApiResult {
    for file_path in paths {
      let key = IVec::try_from(&file_path)?;
      if let Some(_meta) = self
        .inner
        .remove(&key)?
        .map(MetaDataFile::try_from)
        .transpose()?
      {
        // TODO remove files in fs
      }
    }
    Ok(())
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
    let meta: MetaDataFile = Faker.fake();
    ctx
      .state
      .db
      .store(path.clone(), meta.clone())
      .await
      .unwrap();
    let result = ctx.state.db.fetch(&path).unwrap().unwrap();
    assert_eq!(result.created_at, meta.created_at);
    assert_eq!(result.expire_date_time, meta.expire_date_time);
    assert_eq!(result.secret, meta.secret);
    assert_eq!(result.max_download, meta.max_download);
    assert_eq!(result.count_downloads, meta.count_downloads);
  }

  #[test_context(StateTestContext)]
  #[tokio::test]
  async fn test_store_and_update_file(ctx: &mut StateTestContext) {
    let path: FilePath = Faker.fake();
    let meta: MetaDataFile = Faker.fake();
    ctx
      .state
      .db
      .store(path.clone(), meta.clone())
      .await
      .unwrap();
    let mut updated_meta = meta.clone();
    updated_meta.count_downloads += 1;
    ctx
      .state
      .db
      .update(&path, meta.clone(), updated_meta)
      .unwrap();
    let result = ctx.state.db.fetch(&path).unwrap().unwrap();
    assert_eq!(result.created_at, meta.created_at);
    assert_eq!(result.expire_date_time, meta.expire_date_time);
    assert_eq!(result.secret, meta.secret);
    assert_eq!(result.max_download, meta.max_download);
    assert_eq!(result.count_downloads, meta.count_downloads + 1);
  }

  #[test_context(StateTestContext)]
  #[tokio::test]
  async fn test_store_file_and_check_it_existence(ctx: &mut StateTestContext) {
    let path: FilePath = Faker.fake();
    let meta: MetaDataFile = Faker.fake();
    ctx
      .state
      .db
      .store(path.clone(), meta.clone())
      .await
      .unwrap();
    assert!(ctx.state.db.exist(&path).unwrap());
  }

  #[test_context(StateTestContext)]
  #[tokio::test]
  async fn test_store_file_and_expire_it(ctx: &mut StateTestContext) {
    let path: FilePath = Faker.fake();
    let mut meta: MetaDataFile = Faker.fake();
    meta.expire_date_time = Utc::now();
    ctx
      .state
      .db
      .store(path.clone(), meta.clone())
      .await
      .unwrap();
    tokio::time::sleep(Duration::from_secs(1)).await;
    assert!(!ctx.state.db.exist(&path).unwrap());
  }

  #[test_context(StateTestContext)]
  #[tokio::test]
  async fn test_store_file_and_successfully_delete_it(ctx: &mut StateTestContext) {
    let path: FilePath = Faker.fake();
    let meta: MetaDataFile = Faker.fake();
    ctx
      .state
      .db
      .store(path.clone(), meta.clone())
      .await
      .unwrap();
    ctx.state.db.delete(path.clone()).await.unwrap().unwrap();
    assert!(!ctx.state.db.exist(&path).unwrap());
  }

  #[test_context(StateTestContext)]
  #[tokio::test]
  async fn test_delete_file_that_does_not_exist(ctx: &mut StateTestContext) {
    let mut path: FilePath = Faker.fake();
    let meta: MetaDataFile = Faker.fake();
    ctx
      .state
      .db
      .store(path.clone(), meta.clone())
      .await
      .unwrap();
    path.file_name = format!("{}.txt", Faker.fake::<String>());
    let result = ctx.state.db.delete(path).await.unwrap();
    assert!(result.is_none())
  }

  #[test_context(StateTestContext)]
  #[tokio::test]
  async fn test_fetch_file_that_does_not_exist(ctx: &mut StateTestContext) {
    let mut path: FilePath = Faker.fake();
    let meta: MetaDataFile = Faker.fake();
    ctx
      .state
      .db
      .store(path.clone(), meta.clone())
      .await
      .unwrap();
    path.file_name = format!("{}.txt", Faker.fake::<String>());
    let result = ctx.state.db.fetch(&path).unwrap();
    assert!(result.is_none())
  }
}
