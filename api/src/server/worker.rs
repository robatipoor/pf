use async_trait::async_trait;
use axum::extract::State;
use futures_util::future::join_all;
use tokio::task::JoinHandle;
use tracing::{error, info};

use super::ApiState;

#[async_trait]
pub trait ApiTask: Send + Sync {
  const NAME: &'static str;
  fn new(state: State<ApiState>) -> Self;
  async fn run(&self);
}

pub fn spawn(state: State<ApiState>) -> JoinHandle<std::io::Result<()>> {
  tokio::task::spawn(worker(state))
}

async fn worker(state: State<ApiState>) -> std::io::Result<()> {
  let gc = GarbageCollectorTask { state };
  let jhs: Vec<JoinHandle<()>> = vec![doing_job(gc)];
  join_all(jhs).await;
  Ok(())
}

fn doing_job<T: ApiTask + 'static + Send>(task: T) -> JoinHandle<()> {
  tokio::task::spawn(async move {
    info!("*** start task: {} ***", T::NAME);
    task.run().await;
  })
}

pub struct GarbageCollectorTask {
  state: State<ApiState>,
}

#[async_trait::async_trait]
impl ApiTask for GarbageCollectorTask {
  const NAME: &'static str = "GC_TASK";

  fn new(state: State<ApiState>) -> Self {
    Self { state }
  }

  async fn run(&self) {
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
          error!("failed gc task: {e}");
          self.state.db.waiting_for_notify().await;
        }
      }
    }
  }
}
