use crate::error::result::ApiResult;

use super::ApiState;

pub struct GarbageCollectorTask {
  state: ApiState,
}

impl GarbageCollectorTask {
  pub fn new(state: ApiState) -> Self {
    Self { state }
  }

  pub async fn run(self) -> ApiResult {
    let base_dir = self.state.config.fs.base_dir.clone();
    loop {
      match self.state.db.purge(&base_dir).await {
        Ok(Some(d)) => {
          tokio::select! {
            _ = tokio::time::sleep(d) => {},
            _ = self.state.db.waiting_for_notify() => {},
          }
        }
        Ok(None) => {
          self.state.db.waiting_for_notify().await;
        }
        Err(err) => {
          tracing::error!("Failed garbage collector task, Error: {err}");
          self.state.db.waiting_for_notify().await;
        }
      }
    }
  }
}
