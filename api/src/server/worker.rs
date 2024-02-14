use crate::error::ApiResult;

use super::ApiState;

pub struct GarbageCollectorTask {
  state: ApiState,
}

impl GarbageCollectorTask {
  pub fn new(state: ApiState) -> Self {
    Self { state }
  }

  pub async fn run(self) -> ApiResult {
    loop {
      match self.state.db.purge().await {
        Ok(Some(d)) => {
          tokio::select! {
            _ = tokio::time::sleep(d) => {},
            _ = self.state.db.waiting_for_notify() => {},
          }
        }
        Ok(None) => {
          // TODO remove unused files in fs
          self.state.db.waiting_for_notify().await;
        }
        Err(e) => {
          tracing::error!("Failed garbage collector task, Error: {e}");
          self.state.db.waiting_for_notify().await;
        }
      }
    }
  }
}
