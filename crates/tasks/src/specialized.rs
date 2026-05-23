// Design ported from lakekeeper/lakekeeper, Apache-2.0.
// Upstream source: https://github.com/lakekeeper/lakekeeper/blob/05cce5797d3321ce30c59e034f19a3861c39cd88/crates/lakekeeper/src/service/tasks/mod.rs
//
// Adapted for unitycatalog-rs:
//   * Drops the `C: CatalogStore` generic parameter; works against a
//     `&dyn TaskStore` directly.
//   * Auto-commits its own transactions for the convenience methods
//     (`heartbeat`, `record_success`, `record_failure`) since this crate is
//     not coupled to a wider transaction-manager abstraction.

use std::{marker::PhantomData, time::Duration};

use serde::de::DeserializeOwned;
use sqlx::{Postgres, Transaction};
use tokio_util::sync::CancellationToken;

use crate::error::{Result, TaskError};
use crate::store::TaskStore;
use crate::types::{
    ScheduleTaskMetadata, Status, Task, TaskAttemptId, TaskCheckState, TaskConfig, TaskData,
    TaskExecutionDetails, TaskId, TaskInput, TaskIntermediateStatus, TaskMetadata, TaskQueueName,
};

/// Type-safe wrapper around an active `Task` for a specific queue type.
///
/// Workers register against a concrete `SpecializedTask<Q, D, E>` and use its
/// helpers to schedule, poll, heartbeat, and finalise their work. The wrapper
/// owns the typed payload `D` and tracks `E` (execution details) through a
/// `PhantomData` so heartbeats can publish strongly-typed progress info.
#[derive(Debug, Clone, PartialEq)]
pub struct SpecializedTask<Q: TaskConfig, D: TaskData, E: TaskExecutionDetails> {
    pub task_metadata: TaskMetadata,
    pub id: TaskAttemptId,
    pub status: TaskIntermediateStatus,
    pub picked_up_at: Option<chrono::DateTime<chrono::Utc>>,
    pub config: Option<Q>,
    pub data: D,
    execution_details: PhantomData<E>,
}

impl<Q: TaskConfig, D: TaskData, E: TaskExecutionDetails> AsRef<TaskAttemptId>
    for SpecializedTask<Q, D, E>
{
    fn as_ref(&self) -> &TaskAttemptId {
        &self.id
    }
}

impl<Q: TaskConfig, D: TaskData, E: TaskExecutionDetails> SpecializedTask<Q, D, E> {
    /// Static queue name for this specialisation.
    pub fn queue_name() -> &'static TaskQueueName {
        Q::queue_name()
    }

    pub fn task_id(&self) -> TaskId {
        self.id.task_id
    }

    pub fn attempt(&self) -> i32 {
        self.id.attempt
    }

    pub fn id(&self) -> TaskAttemptId {
        self.id
    }

    /// Schedule a single task on this queue in the caller's transaction.
    ///
    /// Returns `Some(task_id)` if the task was enqueued, or `None` if an
    /// active task already exists for the same `(entity, queue)` tuple.
    pub async fn schedule_task(
        store: &dyn TaskStore,
        task_metadata: ScheduleTaskMetadata,
        payload: D,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<Option<TaskId>> {
        let payload = serde_json::to_value(&payload).map_err(|e| TaskError::Serialization {
            kind: "payload",
            queue: Self::queue_name().clone(),
            source: e,
        })?;

        let ids = store
            .enqueue_tasks(
                Self::queue_name(),
                vec![TaskInput {
                    task_metadata,
                    payload,
                }],
                tx,
            )
            .await?;

        Ok(ids.into_iter().next())
    }

    /// Schedule multiple tasks in a single transaction.
    ///
    /// CAUTION: the returned `Vec` may be shorter than `tasks` — inputs whose
    /// `(entity, queue)` tuple was already active are silently skipped.
    pub async fn schedule_tasks(
        store: &dyn TaskStore,
        tasks: impl IntoIterator<Item = (ScheduleTaskMetadata, D)>,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<Vec<TaskId>> {
        let inputs = tasks
            .into_iter()
            .map(|(task_metadata, payload)| {
                Ok(TaskInput {
                    task_metadata,
                    payload: serde_json::to_value(&payload).map_err(|e| {
                        TaskError::Serialization {
                            kind: "payload",
                            queue: Self::queue_name().clone(),
                            source: e,
                        }
                    })?,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        store.enqueue_tasks(Self::queue_name(), inputs, tx).await
    }

    /// Atomically pick the next ready task from this queue.
    ///
    /// On JSON deserialisation failures the task is recorded as failed (so it
    /// won't be picked up again on the next iteration) and `Ok(None)` is
    /// returned to the worker.
    pub async fn pick_new_task(store: &dyn TaskStore) -> Result<Option<Self>> {
        let Some(task) = store
            .pick_new_task(Self::queue_name(), Q::max_time_since_last_heartbeat())
            .await?
        else {
            return Ok(None);
        };

        let data: D = match decode(&task, &task.data, "task data") {
            Ok(v) => v,
            Err(err) => {
                Self::report_deserialization_failure(store, task.id, &err.to_string()).await;
                return Ok(None);
            }
        };

        let config: Option<Q> = match task
            .config
            .as_ref()
            .map(|cfg| decode(&task, cfg, "queue configuration"))
            .transpose()
        {
            Ok(v) => v,
            Err(err) => {
                Self::report_deserialization_failure(store, task.id, &err.to_string()).await;
                return Ok(None);
            }
        };

        Ok(Some(Self {
            task_metadata: task.task_metadata,
            id: task.id,
            status: task.status,
            picked_up_at: task.picked_up_at,
            config,
            data,
            execution_details: PhantomData,
        }))
    }

    /// Long-poll the queue, returning the first available task. Resolves to
    /// `None` if `cancellation_token` is triggered first.
    pub async fn poll_for_new_task(
        store: &dyn TaskStore,
        poll_interval: Duration,
        cancellation_token: CancellationToken,
    ) -> Option<Self> {
        loop {
            tokio::select! {
                () = cancellation_token.cancelled() => {
                    tracing::info!(
                        "Graceful shutdown requested for queue `{}`",
                        Self::queue_name()
                    );
                    return None;
                }
                pick = Self::pick_new_task(store) => {
                    match pick {
                        Ok(Some(task)) => {
                            tracing::debug!("Picked up `{}` task {}.", Self::queue_name(), task.id);
                            return Some(task);
                        }
                        Ok(None) => {
                            // No work — sleep with a bit of jitter to avoid
                            // thundering herd across workers.
                            let jitter = fastrand::u64(0..500);
                            tokio::select! {
                                () = cancellation_token.cancelled() => return None,
                                () = tokio::time::sleep(poll_interval + Duration::from_millis(jitter)) => continue,
                            }
                        }
                        Err(e) => {
                            tracing::error!(
                                "Failed to pick new task from queue `{}`. Retrying in 5s. Error: {e}",
                                Self::queue_name(),
                            );
                            tokio::select! {
                                () = cancellation_token.cancelled() => return None,
                                () = tokio::time::sleep(Duration::from_secs(5)) => continue,
                            }
                        }
                    }
                }
            }
        }
    }

    /// Heartbeat the task inside an existing transaction.
    pub async fn heartbeat_in_transaction(
        &self,
        store: &dyn TaskStore,
        tx: &mut Transaction<'_, Postgres>,
        progress: f32,
        execution_details: Option<E>,
    ) -> Result<TaskCheckState> {
        let execution_details = execution_details
            .map(|d| serde_json::to_value(d))
            .transpose()
            .map_err(|e| TaskError::Serialization {
                kind: "execution details",
                queue: Self::queue_name().clone(),
                source: e,
            })?;

        store
            .check_and_heartbeat_task(self.id, progress, execution_details, tx)
            .await
    }

    /// Heartbeat the task by opening and committing a short-lived transaction.
    pub async fn heartbeat(
        &self,
        store: &dyn TaskStore,
        progress: f32,
        execution_details: Option<E>,
    ) -> Result<TaskCheckState> {
        let mut tx = store.pool().begin().await?;
        let state = self
            .heartbeat_in_transaction(store, &mut tx, progress, execution_details)
            .await?;
        tx.commit().await?;
        Ok(state)
    }

    /// Record successful completion, retrying transient DB errors up to five
    /// times. Failures are logged but never propagated — by the time we get
    /// here the work has already happened.
    pub async fn record_success(&self, store: &dyn TaskStore, details: Option<&str>) {
        let status = Status::Success(details);
        Self::record_with_retry(store, self.id, status, details, None).await;
    }

    /// Record a final failure with the configured `max_retries` budget so the
    /// store can decide whether to reschedule or archive.
    pub async fn record_failure(&self, store: &dyn TaskStore, error: &str) {
        let status = Status::Failure(error, Q::max_retries());
        Self::record_with_retry(store, self.id, status, None, Some(error)).await;
    }

    /// Record success inside the caller's transaction (no retry wrapper).
    pub async fn record_success_in_transaction(
        &self,
        store: &dyn TaskStore,
        tx: &mut Transaction<'_, Postgres>,
        details: Option<&str>,
    ) -> Result<()> {
        store.record_task_success(self.id, details, tx).await
    }

    async fn record_with_retry(
        store: &dyn TaskStore,
        id: TaskAttemptId,
        status: Status<'_>,
        original_success_details: Option<&str>,
        original_error_details: Option<&str>,
    ) {
        for attempt in 1..=5 {
            let result: Result<()> = async {
                let mut tx = store.pool().begin().await?;
                match status {
                    Status::Success(details) => {
                        store.record_task_success(id, details, &mut tx).await?;
                    }
                    Status::Failure(details, max_retries) => {
                        store
                            .record_task_failure(id, details, max_retries, &mut tx)
                            .await?;
                    }
                }
                tx.commit().await?;
                Ok(())
            }
            .await;

            match result {
                Ok(()) => {
                    tracing::debug!(
                        "Recorded {status} for task {id} in queue `{}` on attempt {attempt}",
                        Self::queue_name(),
                    );
                    return;
                }
                Err(e) if attempt < 5 => {
                    tracing::warn!(
                        "Failed to record {status} for task {id} in queue `{}` on attempt {attempt}/5: {e}",
                        Self::queue_name(),
                    );
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to record {status} for task {id} in queue `{}` after 5 attempts: {e}. \
                         Original success details: {success:?}. Original error details: {error:?}",
                        Self::queue_name(),
                        success = original_success_details,
                        error = original_error_details,
                    );
                }
            }
        }
    }

    async fn report_deserialization_failure(store: &dyn TaskStore, id: TaskAttemptId, error: &str) {
        tracing::error!("{error}. TaskID: {id}");

        let mut tx = match store.pool().begin().await {
            Ok(tx) => tx,
            Err(e) => {
                tracing::error!(
                    "Failed to start transaction recording deserialization failure for task {id} in queue `{}`: {e}. Original error: {error}",
                    Self::queue_name(),
                );
                return;
            }
        };

        let msg = format!("Failed to deserialize task data: {error}");
        if let Err(e) = store
            .record_task_failure(id, &msg, Q::max_retries(), &mut tx)
            .await
        {
            tracing::error!(
                "Failed to record deserialization failure for task {id} in queue `{}`: {e}. Original error: {error}",
                Self::queue_name(),
            );
            return;
        }

        if let Err(e) = tx.commit().await {
            tracing::error!(
                "Failed to commit transaction recording deserialization failure for task {id} in queue `{}`: {e}. Original error: {error}",
                Self::queue_name(),
            );
        }
    }
}

fn decode<T: DeserializeOwned>(
    task: &Task,
    value: &serde_json::Value,
    kind: &'static str,
) -> Result<T> {
    serde_json::from_value(value.clone()).map_err(|e| TaskError::Deserialization {
        kind,
        queue: task.queue_name.clone(),
        source: e,
    })
}
