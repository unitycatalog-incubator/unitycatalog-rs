// Design ported from lakekeeper/lakekeeper, Apache-2.0.
// Upstream source: https://github.com/lakekeeper/lakekeeper/blob/05cce5797d3321ce30c59e034f19a3861c39cd88/crates/lakekeeper/src/service/catalog_store/tasks.rs
//
// Adapted for unitycatalog-rs:
//   * Renamed from `CatalogTaskOps` to `TaskStore` since this crate doesn't
//     model the wider catalog store.
//   * Trait is Postgres-bound (`sqlx::Transaction<'_, sqlx::Postgres>`) to
//     keep it dyn-safe without an associated `Transaction` type. The
//     framework explicitly targets Postgres only — see crate README.
//   * Methods returning lists of `Option<TaskId>` mirror lakekeeper's
//     "deduped enqueue" semantics: positions corresponding to existing active
//     tasks return `None`.

use async_trait::async_trait;
use chrono::Duration;
use sqlx::{PgPool, Postgres, Transaction};

use crate::error::Result;
use crate::types::{Task, TaskAttemptId, TaskCheckState, TaskId, TaskInput, TaskQueueName};

/// Storage primitives for the task queue framework.
///
/// Implementations of this trait drive the actual Postgres queue. All
/// transactional methods accept a borrowed `&mut sqlx::Transaction`, allowing
/// callers (typically `SpecializedTask`) to combine task enqueue/update with
/// other business writes in a single transaction.
#[async_trait]
pub trait TaskStore: Send + Sync + 'static {
    /// Atomically pick the next ready task for the given queue, marking it
    /// `running`. Returns `None` if no task is currently ready.
    ///
    /// `default_max_time_since_last_heartbeat` is the fallback timeout used
    /// to reclaim stale `running` tasks when there is no per-queue override
    /// configured in `task_config`.
    async fn pick_new_task(
        &self,
        queue_name: &TaskQueueName,
        default_max_time_since_last_heartbeat: Duration,
    ) -> Result<Option<Task>>;

    /// Insert tasks into the queue inside the caller's transaction.
    ///
    /// Tasks are deduplicated on the
    /// `(entity_type, entity_id, queue_name)` tuple: re-enqueueing while an
    /// active task already exists is a no-op for that input. The returned
    /// vector contains only the task IDs that were actually inserted, so it
    /// may be shorter than `inputs`.
    async fn enqueue_tasks(
        &self,
        queue_name: &TaskQueueName,
        inputs: Vec<TaskInput>,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<Vec<TaskId>>;

    /// Update the task's heartbeat / progress and return whether the worker
    /// should keep going, stop cleanly, or abort.
    async fn check_and_heartbeat_task(
        &self,
        id: TaskAttemptId,
        progress: f32,
        execution_details: Option<serde_json::Value>,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<TaskCheckState>;

    /// Mark a task attempt as a final success, archiving it into `task_log`
    /// and removing it from the active `task` table.
    async fn record_task_success(
        &self,
        id: TaskAttemptId,
        details: Option<&str>,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<()>;

    /// Record a failed attempt. If `attempt < max_retries`, the task is
    /// returned to the `scheduled` state for another try; otherwise it is
    /// archived to `task_log` as `failed`.
    async fn record_task_failure(
        &self,
        id: TaskAttemptId,
        details: &str,
        max_retries: i32,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<()>;

    /// Borrow the underlying connection pool.
    ///
    /// Used by `SpecializedTask`'s convenience helpers (`heartbeat`,
    /// `record_success`, `record_failure`) to open their own transactions
    /// when callers don't want to thread one through.
    fn pool(&self) -> &PgPool;
}
