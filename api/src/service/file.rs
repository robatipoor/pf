use axum::extract::BodyStream;
use axum_extra::body::AsyncReadBody;
use common::error::ApiResult;
use futures_util::TryStreamExt;
use tokio::fs::File;
use tokio::io::BufWriter;
use tokio_util::io::StreamReader;

use crate::server::ApiState;

pub async fn store(state: &ApiState, file_name: &str) -> ApiResult<String> {
  let code = "";
  let path = format!("{code}/{file_name}");
  if !state.db.exist(&path).await {}
  todo!()
}

pub async fn fetch(state: &ApiState, code: &str, file_name: &str) -> ApiResult<()> {
  let path = format!("{code}/{file_name}");
  if let Some(mut meta) = state.db.find(&path).await {
    if let Some(max) = meta.max_download {
      if max <= 1 {
        state.db.delete(&path).await;
      } else {
        meta.max_download = Some(max - 1);
        state.db.store(path, meta).await?;
      }
    }
  }
  Ok(())
}

pub async fn delete(state: &ApiState, code: &str, file_name: &str) -> ApiResult<()> {
  let path = format!("{code}/{file_name}");
  if let Some(info) = state.db.find(&path).await {
    if info.is_deleteable {
      state.db.delete(&path).await;
    } else {
      //TODO
    }
  }
  Ok(())
}
