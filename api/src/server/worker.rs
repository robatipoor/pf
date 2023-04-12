use async_trait::async_trait;
use axum::extract::State;
use futures_util::future::join_all;
use tokio::task::JoinHandle;
use tracing::info;

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
      if let Ok(Some(d)) = self.state.db.purge().await {
        tokio::time::sleep(d).await;
        // TODO
      } else {
      }
    }
  }
}
