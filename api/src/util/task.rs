use tracing::error;

use crate::error::{result::ApiResult, ApiError};

/// If a task is fail fast after encounter an error node goes down.
pub type IsFailFast = bool;
pub type ApiTask = (
  &'static str,
  IsFailFast,
  futures_util::future::BoxFuture<'static, ApiResult>,
);

pub async fn join_all(tasks: Vec<ApiTask>) -> ApiResult {
  let (sender, mut receiver) = tokio::sync::mpsc::channel::<ApiError>(1);
  for (name, is_fail_fast, task) in tasks {
    let sender = if is_fail_fast {
      Some(sender.clone())
    } else {
      None
    };
    tokio::spawn(async move {
      tracing::info!("Task {name} started.");
      if let Err(e) = task.await {
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
