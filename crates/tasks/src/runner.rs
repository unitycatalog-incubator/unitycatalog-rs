// Design ported from lakekeeper/lakekeeper, Apache-2.0.
// Upstream source: https://github.com/lakekeeper/lakekeeper/blob/05cce5797d3321ce30c59e034f19a3861c39cd88/crates/lakekeeper/src/service/tasks/task_queues_runner.rs

use std::{collections::HashMap, sync::Arc};

use futures::future::BoxFuture;
use tokio_util::sync::CancellationToken;

use crate::types::TaskQueueName;

/// Infinitely running task worker loop. Workers receive a cancellation token
/// and are expected to terminate cooperatively when it's triggered.
pub type TaskQueueWorkerFn =
    Arc<dyn Fn(CancellationToken) -> BoxFuture<'static, ()> + Send + Sync + 'static>;

#[derive(Clone)]
pub(crate) struct QueueWorkerConfig {
    pub worker_fn: TaskQueueWorkerFn,
    pub num_workers: usize,
}

impl std::fmt::Debug for QueueWorkerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueueWorkerConfig")
            .field("worker_fn", &"Fn(...)")
            .field("num_workers", &self.num_workers)
            .finish()
    }
}

/// Spawns and supervises all registered task-queue workers.
///
/// Worker functions are spawned on the current tokio runtime. If a worker
/// finishes unexpectedly (including panics) before cancellation is requested,
/// it is restarted from the same `TaskQueueWorkerFn`. Once the cancellation
/// token is triggered the supervisor stops restarting workers and waits for
/// the survivors to drain.
#[derive(Debug, Clone)]
pub struct TaskQueuesRunner {
    pub(crate) registered_queues: Arc<HashMap<&'static TaskQueueName, QueueWorkerConfig>>,
    pub(crate) cancellation_token: CancellationToken,
}

struct WorkerInfo {
    queue_name: &'static TaskQueueName,
    worker_id: usize,
    handle: tokio::task::JoinHandle<()>,
}

impl TaskQueuesRunner {
    /// Run all registered queue workers until every worker has terminated.
    ///
    /// When `restart_workers` is true, workers that finish before cancellation
    /// is requested are automatically respawned.
    pub async fn run_queue_workers(self, restart_workers: bool) {
        let mut workers = Vec::new();
        let registered_queues = Arc::clone(&self.registered_queues);

        for (queue_name, queue) in registered_queues.iter() {
            tracing::info!(
                "Starting {} workers for task queue `{queue_name}`.",
                queue.num_workers,
            );
            for worker_id in 0..queue.num_workers {
                let task_fn = Arc::clone(&queue.worker_fn);
                let cancellation_token_clone = self.cancellation_token.clone();
                tracing::debug!(
                    "Starting `{queue_name}` worker {} ({}/{})",
                    worker_id,
                    worker_id + 1,
                    queue.num_workers
                );
                workers.push(WorkerInfo {
                    queue_name,
                    worker_id,
                    handle: tokio::task::spawn(task_fn(cancellation_token_clone)),
                });
            }
        }

        loop {
            if workers.is_empty() {
                return;
            }

            let handles: Vec<_> = workers.iter_mut().map(|w| &mut w.handle).collect();
            let (result, index, _) = futures::future::select_all(handles).await;
            let worker = workers.swap_remove(index);

            let log_msg_suffix = if restart_workers {
                "Restarting worker"
            } else {
                "Restarting worker disabled"
            };

            match result {
                Ok(()) if !self.cancellation_token.is_cancelled() => tracing::warn!(
                    "Task queue `{}` worker {} finished. {log_msg_suffix}",
                    worker.queue_name,
                    worker.worker_id,
                ),
                Ok(()) => tracing::info!(
                    "Task queue `{}` worker {} finished gracefully after cancellation.",
                    worker.queue_name,
                    worker.worker_id,
                ),
                Err(e) => {
                    if e.is_panic() {
                        tracing::error!(
                            ?e,
                            "Task queue `{}` worker {} panicked. {log_msg_suffix}",
                            worker.queue_name,
                            worker.worker_id,
                        );
                    } else if e.is_cancelled() {
                        tracing::warn!(
                            ?e,
                            "Task queue `{}` worker {} was cancelled.",
                            worker.queue_name,
                            worker.worker_id,
                        );
                    } else {
                        tracing::error!(
                            ?e,
                            "Task queue `{}` worker {} failed to join. {log_msg_suffix}",
                            worker.queue_name,
                            worker.worker_id,
                        );
                    }
                }
            }

            if restart_workers && !self.cancellation_token.is_cancelled() {
                if let Some(queue) = registered_queues.get(worker.queue_name) {
                    let task_fn = Arc::clone(&queue.worker_fn);
                    let cancellation_token_clone = self.cancellation_token.clone();
                    tracing::debug!(
                        "Restarting task queue `{}` worker {}",
                        worker.queue_name,
                        worker.worker_id,
                    );
                    workers.push(WorkerInfo {
                        queue_name: worker.queue_name,
                        worker_id: worker.worker_id,
                        handle: tokio::task::spawn(task_fn(cancellation_token_clone)),
                    });
                }
            } else if self.cancellation_token.is_cancelled() {
                tracing::info!(
                    "Cancellation requested, not restarting task queue `{}` worker {}",
                    worker.queue_name,
                    worker.worker_id,
                );
            }
        }
    }
}
