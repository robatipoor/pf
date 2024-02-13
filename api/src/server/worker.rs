use async_trait::async_trait;
use axum::extract::State;
use tokio::task::JoinHandle;
use tracing::{error, info};

use crate::error::{ApiError, ApiResult};

use super::ApiState;

#[async_trait]
pub trait ApiTask: Send + Sync {
  async fn run(&self) -> ApiResult;
  fn is_fail_fast(&self) -> bool;
  fn name(&self) -> &'static str;
}

pub fn spawn(state: State<ApiState>) -> JoinHandle<ApiResult> {
  tokio::task::spawn(start(state))
}

pub async fn start(state: State<ApiState>) -> ApiResult {
  let tasks: Vec<Box<dyn ApiTask>> = vec![Box::new(GarbageCollectorTask { state })];
  join_all(tasks).await
}

async fn join_all(tasks: Vec<Box<dyn ApiTask + 'static>>) -> ApiResult {
  let (sender, mut receiver) = tokio::sync::mpsc::channel::<ApiError>(1);

  for task in tasks.into_iter() {
    let sender = if task.is_fail_fast() {
      Some(sender.clone())
    } else {
      None
    };

    tokio::spawn(async move {
      info!("Run task: {}", task.name());
      if let Err(e) = task.run().await {
        if let Some(sender) = sender {
          sender
            .send(e)
            .await
            .unwrap_or_else(|_| unreachable!("This channel never closed."));
        } else {
          error!("A task failed: {e}.");
        }
      }
    });
  }

  match receiver.recv().await {
    Some(err) => Err(err),
    None => unreachable!("This channel never closed."),
  }
}

pub struct GarbageCollectorTask {
  state: State<ApiState>,
}

#[async_trait::async_trait]
impl ApiTask for GarbageCollectorTask {
  async fn run(&self) -> ApiResult {
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
          error!("Failed garbage collector task, Error: {e}");
          self.state.db.waiting_for_notify().await;
        }
      }
    }
  }

  fn is_fail_fast(&self) -> bool {
    false
  }

  fn name(&self) -> &'static str {
    "GC_TASK"
  }
}
