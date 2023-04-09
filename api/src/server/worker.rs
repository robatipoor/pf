use async_trait::async_trait;
use axum::extract::State;
use common::error::TaskError;
use futures_util::future::join_all;
use std::time::Duration;
use tokio::task::JoinHandle;
use tracing::{error, info};

use crate::config::WorkerConfig;

use super::ApiState;

#[derive(Debug)]
pub struct TaskResult {
  pub delay: u64,
  pub status: TaskStatus,
}

#[derive(Debug)]
pub enum TaskStatus {
  Completed,
  QueueEmpty,
}

#[async_trait]
pub trait ApiTask: Send + Sync {
  const NAME: &'static str;
  fn new(state: State<ApiState>) -> Self;
  async fn run(&self) -> Result<TaskResult, TaskError>;
}

pub fn spawn(state: State<ApiState>) -> JoinHandle<std::io::Result<()>> {
  tokio::task::spawn(worker(state))
}

async fn worker(state: State<ApiState>) -> std::io::Result<()> {
  let jhs: Vec<JoinHandle<()>> = vec![doing_job(state.config.worker.clone(), GarbageCollectorTask)];
  join_all(jhs).await;
  Ok(())
}

fn doing_job<T: ApiTask + 'static + Send>(config: WorkerConfig, task: T) -> JoinHandle<()> {
  tokio::task::spawn(async move {
    info!("*** start task: {} ***", T::NAME);
    loop {
      match task.run().await {
        Ok(r) => match r.status {
          TaskStatus::Completed => {
            info!("job: {} complete", T::NAME);
            tokio::time::sleep(Duration::from_secs(r.delay)).await
          }
          TaskStatus::QueueEmpty => {
            info!("job: {} queue is empty", T::NAME);
            tokio::time::sleep(Duration::from_secs(r.delay)).await
          }
        },
        Err(e) => {
          error!("job: {} faild error message: {e:?}", T::NAME);
          tokio::time::sleep(Duration::from_secs(config.failed_task_delay)).await
        }
      }
    }
  })
}

pub struct GarbageCollectorTask;

#[async_trait::async_trait]
impl ApiTask for GarbageCollectorTask {
  const NAME: &'static str = "GC_TASK";

  fn new(state: State<ApiState>) -> Self {
    Self
  }

  async fn run(&self) -> Result<TaskResult, TaskError> {
    Ok(TaskResult {
      delay: 100000,
      status: TaskStatus::Completed,
    })
  }
}
