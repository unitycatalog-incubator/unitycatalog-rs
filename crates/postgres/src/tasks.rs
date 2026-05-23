//! Postgres implementation of [`unitycatalog_tasks::TaskStore`].
//!
//! The SQL queries below are ported from
//! [lakekeeper/lakekeeper](https://github.com/lakekeeper/lakekeeper) (Apache-2.0),
//! specifically
//! <https://github.com/lakekeeper/lakekeeper/blob/05cce5797d3321ce30c59e034f19a3861c39cd88/crates/lakekeeper/src/implementations/postgres/tasks.rs>.
//!
//! Notable adaptations:
//!   * No `warehouse_id` / `project_id` joins (UC has neither).
//!   * `entity_type` is stored as `text`, not a closed enum, so the framework
//!     stays generic.
//!   * Queries are written with the *dynamic* `sqlx::query` / `sqlx::query_as`
//!     APIs rather than the compile-time-checked macros so we can ship the
//!     migration without immediately regenerating the workspace `.sqlx`
//!     offline cache. Integration tests cover correctness against a real DB.

use async_trait::async_trait;
use chrono::Duration;
use sqlx::postgres::types::PgInterval;
use sqlx::{PgPool, Postgres, Row, Transaction};
use unitycatalog_tasks::{
    Result as TaskResult, Task, TaskAttemptId, TaskCheckState, TaskEntity, TaskError, TaskId,
    TaskInput, TaskIntermediateStatus, TaskMetadata, TaskQueueName, TaskStore,
};
use uuid::Uuid;

use crate::GraphStore;

#[async_trait]
impl TaskStore for GraphStore {
    async fn pick_new_task(
        &self,
        queue_name: &TaskQueueName,
        default_max_time_since_last_heartbeat: Duration,
    ) -> TaskResult<Option<Task>> {
        let interval = duration_to_pg_interval(default_max_time_since_last_heartbeat)?;

        let row = sqlx::query(PICK_TASK_SQL)
            .bind(queue_name.as_str())
            .bind(interval)
            .fetch_optional(&self.pool)
            .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        Ok(Some(row_to_task(&row)?))
    }

    async fn enqueue_tasks(
        &self,
        queue_name: &TaskQueueName,
        inputs: Vec<TaskInput>,
        tx: &mut Transaction<'_, Postgres>,
    ) -> TaskResult<Vec<TaskId>> {
        if inputs.is_empty() {
            return Ok(Vec::new());
        }

        let mut task_ids = Vec::with_capacity(inputs.len());
        let mut parent_task_ids = Vec::with_capacity(inputs.len());
        let mut scheduled_fors = Vec::with_capacity(inputs.len());
        let mut payloads = Vec::with_capacity(inputs.len());
        let mut entity_types: Vec<Option<String>> = Vec::with_capacity(inputs.len());
        let mut entity_ids: Vec<Option<Uuid>> = Vec::with_capacity(inputs.len());
        // entity_name is a TEXT[] per row in the DB; we serialise as JSON
        // arrays for the `unnest` of `jsonb[]` then convert back inside SQL.
        let mut entity_names_json: Vec<Option<serde_json::Value>> =
            Vec::with_capacity(inputs.len());

        for input in inputs {
            let TaskInput {
                task_metadata,
                payload,
            } = input;
            let (entity_type, entity_id, entity_name) = match task_metadata.entity {
                TaskEntity::System => (None, None, None),
                TaskEntity::Resource {
                    entity_type,
                    entity_id,
                    entity_name,
                } => (
                    Some(entity_type),
                    Some(entity_id),
                    Some(serde_json::Value::Array(
                        entity_name
                            .into_iter()
                            .map(serde_json::Value::String)
                            .collect(),
                    )),
                ),
            };

            task_ids.push(Uuid::now_v7());
            parent_task_ids.push(task_metadata.parent_task_id.map(Uuid::from));
            scheduled_fors.push(task_metadata.scheduled_for);
            payloads.push(payload);
            entity_types.push(entity_type);
            entity_ids.push(entity_id);
            entity_names_json.push(entity_name);
        }

        let rows = sqlx::query(ENQUEUE_TASKS_SQL)
            .bind(&task_ids)
            .bind(queue_name.as_str())
            .bind(&parent_task_ids)
            .bind(&scheduled_fors)
            .bind(&payloads)
            .bind(&entity_types)
            .bind(&entity_ids)
            .bind(&entity_names_json)
            .bind(TaskIntermediateStatus::Scheduled)
            .fetch_all(&mut **tx)
            .await?;

        let mut inserted = Vec::with_capacity(rows.len());
        for row in rows {
            let id: Uuid = row.try_get("task_id")?;
            inserted.push(TaskId::from(id));
        }
        Ok(inserted)
    }

    async fn check_and_heartbeat_task(
        &self,
        id: TaskAttemptId,
        progress: f32,
        execution_details: Option<serde_json::Value>,
        tx: &mut Transaction<'_, Postgres>,
    ) -> TaskResult<TaskCheckState> {
        let TaskAttemptId { task_id, attempt } = id;
        let row = sqlx::query(HEARTBEAT_SQL)
            .bind(*task_id)
            .bind(progress)
            .bind(execution_details)
            .bind(attempt)
            .fetch_optional(&mut **tx)
            .await?;

        let Some(row) = row else {
            return Ok(TaskCheckState::NotActive);
        };
        let status: TaskIntermediateStatus = row.try_get("status")?;
        Ok(match status {
            TaskIntermediateStatus::ShouldStop | TaskIntermediateStatus::Scheduled => {
                TaskCheckState::Stop
            }
            TaskIntermediateStatus::Running => TaskCheckState::Continue,
        })
    }

    async fn record_task_success(
        &self,
        id: TaskAttemptId,
        details: Option<&str>,
        tx: &mut Transaction<'_, Postgres>,
    ) -> TaskResult<()> {
        let TaskAttemptId { task_id, attempt } = id;
        let row = sqlx::query(RECORD_SUCCESS_SQL)
            .bind(*task_id)
            .bind(details)
            .bind(attempt)
            .fetch_one(&mut **tx)
            .await?;

        let task_log_exists: bool = row.try_get("task_log_exists")?;
        let log_inserted: bool = row.try_get("log_inserted")?;

        if !task_log_exists {
            return Err(TaskError::TaskNotFound { id });
        }
        if !log_inserted {
            return Err(TaskError::AttemptAlreadyRecorded {
                id,
                outcome: "success",
            });
        }
        Ok(())
    }

    async fn record_task_failure(
        &self,
        id: TaskAttemptId,
        details: &str,
        max_retries: i32,
        tx: &mut Transaction<'_, Postgres>,
    ) -> TaskResult<()> {
        let TaskAttemptId { task_id, attempt } = id;
        let row = if attempt >= max_retries {
            sqlx::query(RECORD_TERMINAL_FAILURE_SQL)
                .bind(*task_id)
                .bind(attempt)
                .bind(details)
                .fetch_one(&mut **tx)
                .await?
        } else {
            sqlx::query(RECORD_RETRY_FAILURE_SQL)
                .bind(*task_id)
                .bind(details)
                .bind(attempt)
                .fetch_one(&mut **tx)
                .await?
        };

        let task_log_exists: bool = row.try_get("task_log_exists")?;
        let log_inserted: bool = row.try_get("log_inserted")?;

        if !task_log_exists {
            return Err(TaskError::TaskNotFound { id });
        }
        if !log_inserted {
            return Err(TaskError::AttemptAlreadyRecorded {
                id,
                outcome: "failure",
            });
        }
        Ok(())
    }

    fn pool(&self) -> &PgPool {
        &self.pool
    }
}

fn duration_to_pg_interval(d: Duration) -> TaskResult<PgInterval> {
    let microseconds = d.num_microseconds().ok_or_else(|| {
        TaskError::internal("duration overflowed when converting to microseconds")
    })?;
    Ok(PgInterval {
        months: 0,
        days: 0,
        microseconds,
    })
}

fn row_to_task(row: &sqlx::postgres::PgRow) -> TaskResult<Task> {
    let task_id: Uuid = row.try_get("task_id")?;
    let queue_name: String = row.try_get("queue_name")?;
    let status: TaskIntermediateStatus = row.try_get("status")?;
    let parent_task_id: Option<Uuid> = row.try_get("parent_task_id")?;
    let scheduled_for: chrono::DateTime<chrono::Utc> = row.try_get("scheduled_for")?;
    let picked_up_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("picked_up_at")?;
    let attempt: i32 = row.try_get("attempt")?;
    let task_data: serde_json::Value = row.try_get("task_data")?;
    let entity_type: Option<String> = row.try_get("entity_type")?;
    let entity_id: Option<Uuid> = row.try_get("entity_id")?;
    let entity_name: Option<Vec<String>> = row.try_get("entity_name")?;
    let config: Option<serde_json::Value> = row.try_get("config")?;

    let entity = match (entity_type, entity_id, entity_name) {
        (Some(entity_type), Some(entity_id), Some(entity_name)) => TaskEntity::Resource {
            entity_type,
            entity_id,
            entity_name,
        },
        (None, None, None) => TaskEntity::System,
        _ => {
            return Err(TaskError::internal(
                "task row has partial entity fields (entity_type / entity_id / entity_name out of sync)",
            ));
        }
    };

    Ok(Task {
        task_metadata: TaskMetadata {
            parent_task_id: parent_task_id.map(TaskId::from),
            scheduled_for,
            entity,
        },
        queue_name: TaskQueueName::from(queue_name),
        id: TaskAttemptId {
            task_id: TaskId::from(task_id),
            attempt,
        },
        status,
        picked_up_at,
        config,
        data: task_data,
    })
}

// -------------------------------------------------------------- SQL ---------

/// FOR UPDATE SKIP LOCKED picker that also reclaims stale `running` tasks
/// whose last heartbeat is older than the configured timeout.
const PICK_TASK_SQL: &str = r#"
WITH picked_task AS (
    SELECT t.*, tc.config
    FROM task t
    LEFT JOIN task_config tc ON tc.queue_name = t.queue_name
    WHERE t.queue_name = $1
        AND t.scheduled_for <= now()
        AND (
            t.status = 'scheduled'
            OR (t.status != 'scheduled'
                AND (now() - t.last_heartbeat_at) > COALESCE(tc.max_time_since_last_heartbeat, $2))
        )
    FOR UPDATE OF t SKIP LOCKED
    LIMIT 1
),
inserted AS (
    INSERT INTO task_log(
        task_id, queue_name, task_data, status, entity_id, entity_type,
        entity_name, message, attempt, started_at, duration, progress,
        execution_details, attempt_scheduled_for, last_heartbeat_at,
        parent_task_id, task_created_at
    )
    SELECT
        task_id, queue_name, task_data, 'failed', entity_id, entity_type,
        entity_name, 'Attempt timed out.', attempt, picked_up_at,
        now() - picked_up_at, progress, execution_details, scheduled_for,
        last_heartbeat_at, parent_task_id, created_at
    FROM picked_task
    WHERE status != 'scheduled'
    ON CONFLICT (task_id, attempt) DO NOTHING
)
UPDATE task
SET status = 'running',
    progress = 0.0,
    execution_details = NULL,
    picked_up_at = now(),
    last_heartbeat_at = now(),
    attempt = task.attempt + 1
FROM picked_task p
WHERE task.task_id = p.task_id AND task.attempt = p.attempt
RETURNING
    task.task_id,
    task.queue_name,
    task.status,
    task.parent_task_id,
    task.scheduled_for,
    task.picked_up_at,
    task.attempt,
    task.task_data,
    task.entity_type,
    task.entity_id,
    task.entity_name,
    (SELECT config FROM picked_task) AS config
"#;

/// Bulk INSERT using `unnest` arrays for the variable-length columns.
///
/// `$8` is `jsonb[]` of either `NULL` (system task) or a JSON array of strings
/// (the resource entity name). We project it back to `text[]` inside SQL.
const ENQUEUE_TASKS_SQL: &str = r#"
INSERT INTO task(
    task_id, queue_name, status, parent_task_id, scheduled_for, task_data,
    entity_type, entity_id, entity_name
)
SELECT
    t.task_id,
    $2,
    $9,
    t.parent_task_id,
    COALESCE(t.scheduled_for, now()),
    t.payload,
    t.entity_type,
    t.entity_id,
    CASE
        WHEN t.entity_name IS NULL THEN NULL
        ELSE ARRAY(SELECT jsonb_array_elements_text(t.entity_name))
    END
FROM unnest(
    $1::uuid[],
    $3::uuid[],
    $4::timestamptz[],
    $5::jsonb[],
    $6::text[],
    $7::uuid[],
    $8::jsonb[]
) AS t(task_id, parent_task_id, scheduled_for, payload, entity_type, entity_id, entity_name)
ON CONFLICT (entity_type, entity_id, queue_name) DO NOTHING
RETURNING task_id
"#;

/// Update `last_heartbeat_at`, `progress`, `execution_details`. Returning the
/// current status lets the worker react to external stop signals.
const HEARTBEAT_SQL: &str = r#"
WITH heartbeat AS (
    UPDATE task
    SET last_heartbeat_at = now(),
        progress = $2,
        execution_details = $3
    WHERE task_id = $1 AND attempt = $4
    RETURNING status
)
SELECT status FROM heartbeat
"#;

/// Move the task to `task_log` as `success` and delete from the active table.
/// Reports both whether the original row existed (to surface "not found") and
/// whether the log row was actually inserted (to detect already-recorded
/// attempts).
const RECORD_SUCCESS_SQL: &str = r#"
WITH moved AS (
    DELETE FROM task WHERE task_id = $1 AND attempt = $3 RETURNING *
),
existing_log AS (
    SELECT 1 FROM task_log WHERE task_id = $1 AND attempt = $3
),
log_insert AS (
    INSERT INTO task_log(
        task_id, queue_name, task_data, status, entity_id, entity_type,
        entity_name, message, attempt, started_at, duration, progress,
        execution_details, attempt_scheduled_for, last_heartbeat_at,
        parent_task_id, task_created_at
    )
    SELECT
        task_id, queue_name, task_data, 'success', entity_id, entity_type,
        entity_name, $2, attempt, picked_up_at, now() - picked_up_at, 1.0,
        execution_details, scheduled_for, last_heartbeat_at, parent_task_id,
        created_at
    FROM moved
    ON CONFLICT (task_id, attempt) DO NOTHING
    RETURNING task_id
)
SELECT
    (EXISTS(SELECT 1 FROM existing_log) OR EXISTS(SELECT 1 FROM moved)) AS task_log_exists,
    EXISTS(SELECT 1 FROM log_insert) AS log_inserted
"#;

/// Final failure: archive into `task_log` and delete from `task`.
const RECORD_TERMINAL_FAILURE_SQL: &str = r#"
WITH moved AS (
    DELETE FROM task WHERE task_id = $1 AND attempt = $2 RETURNING *
),
existing_log AS (
    SELECT 1 FROM task_log WHERE task_id = $1 AND attempt = $2
),
log_insert AS (
    INSERT INTO task_log(
        task_id, queue_name, task_data, status, entity_id, entity_type,
        entity_name, message, attempt, started_at, duration, progress,
        execution_details, attempt_scheduled_for, last_heartbeat_at,
        parent_task_id, task_created_at
    )
    SELECT
        task_id, queue_name, task_data, 'failed', entity_id, entity_type,
        entity_name, $3, attempt, picked_up_at, now() - picked_up_at,
        progress, execution_details, scheduled_for, last_heartbeat_at,
        parent_task_id, created_at
    FROM moved
    ON CONFLICT (task_id, attempt) DO NOTHING
    RETURNING task_id
)
SELECT
    (EXISTS(SELECT 1 FROM existing_log) OR EXISTS(SELECT 1 FROM moved)) AS task_log_exists,
    EXISTS(SELECT 1 FROM log_insert) AS log_inserted
"#;

/// Retryable failure: archive the attempt to `task_log` and reset the task
/// back to `scheduled` so it can be picked up again.
const RECORD_RETRY_FAILURE_SQL: &str = r#"
WITH locked AS (
    SELECT * FROM task WHERE task_id = $1 AND attempt = $3 FOR UPDATE
),
existing_log AS (
    SELECT 1 FROM task_log WHERE task_id = $1 AND attempt = $3
),
log_insert AS (
    INSERT INTO task_log(
        task_id, queue_name, task_data, status, entity_id, entity_type,
        entity_name, message, attempt, started_at, duration, progress,
        execution_details, attempt_scheduled_for, last_heartbeat_at,
        parent_task_id, task_created_at
    )
    SELECT
        task_id, queue_name, task_data, 'failed', entity_id, entity_type,
        entity_name, $2, attempt, picked_up_at, now() - picked_up_at,
        progress, execution_details, scheduled_for, last_heartbeat_at,
        parent_task_id, created_at
    FROM locked
    ON CONFLICT (task_id, attempt) DO NOTHING
    RETURNING task_id
),
task_update AS (
    UPDATE task t
    SET status = 'scheduled',
        progress = 0.0,
        picked_up_at = NULL,
        execution_details = NULL,
        last_heartbeat_at = NULL
    FROM locked
    WHERE t.task_id = locked.task_id AND t.attempt = locked.attempt
    RETURNING t.task_id
)
SELECT
    (EXISTS(SELECT 1 FROM existing_log) OR EXISTS(SELECT 1 FROM log_insert)) AS task_log_exists,
    EXISTS(SELECT 1 FROM log_insert) AS log_inserted
"#;

#[cfg(all(test, feature = "integration-pg"))]
mod tests {
    use std::time::Duration as StdDuration;

    use chrono::Duration;
    use serde_json::json;
    use sqlx::PgPool;
    use unitycatalog_tasks::{
        ScheduleTaskMetadata, TaskEntity, TaskInput, TaskQueueName, TaskStore,
    };
    use uuid::Uuid;

    use super::*;
    use crate::GraphStore;

    fn store(pool: PgPool) -> GraphStore {
        GraphStore::new(pool, None)
    }

    fn queue() -> TaskQueueName {
        TaskQueueName::from("test-queue")
    }

    fn make_input(entity_uuid: Uuid) -> TaskInput {
        TaskInput {
            task_metadata: ScheduleTaskMetadata {
                parent_task_id: None,
                scheduled_for: None,
                entity: TaskEntity::Resource {
                    entity_type: "table".into(),
                    entity_id: entity_uuid,
                    entity_name: vec!["cat".into(), "schema".into(), "tbl".into()],
                },
            },
            payload: json!({"foo": "bar"}),
        }
    }

    #[sqlx::test]
    async fn enqueue_dedup(pool: PgPool) {
        let store = store(pool);
        let entity = Uuid::new_v4();

        let mut tx = store.pool().begin().await.unwrap();
        let ids1 = store
            .enqueue_tasks(&queue(), vec![make_input(entity)], &mut tx)
            .await
            .unwrap();
        let ids2 = store
            .enqueue_tasks(&queue(), vec![make_input(entity)], &mut tx)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        assert_eq!(ids1.len(), 1);
        assert_eq!(ids2.len(), 0, "duplicate (entity, queue) should be skipped");
    }

    #[sqlx::test]
    async fn pick_skip_locked_picks_one(pool: PgPool) {
        let store = store(pool);

        let mut tx = store.pool().begin().await.unwrap();
        let _ = store
            .enqueue_tasks(&queue(), vec![make_input(Uuid::new_v4())], &mut tx)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let picked = store
            .pick_new_task(&queue(), Duration::seconds(300))
            .await
            .unwrap()
            .expect("a task should be picked");

        assert_eq!(picked.attempt(), 1, "picking increments attempt to 1");
        assert_eq!(picked.queue_name.as_str(), "test-queue");
    }

    #[sqlx::test]
    async fn pick_reclaims_stale_running(pool: PgPool) {
        let store = store(pool);

        let mut tx = store.pool().begin().await.unwrap();
        let _ = store
            .enqueue_tasks(&queue(), vec![make_input(Uuid::new_v4())], &mut tx)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        // First pick marks the task as running.
        let first = store
            .pick_new_task(&queue(), Duration::milliseconds(100))
            .await
            .unwrap()
            .expect("task picked initially");

        // Wait past the heartbeat timeout so the next pick reclaims it.
        tokio::time::sleep(StdDuration::from_millis(300)).await;

        let second = store
            .pick_new_task(&queue(), Duration::milliseconds(100))
            .await
            .unwrap()
            .expect("stale task should be re-picked");

        assert_eq!(first.task_id(), second.task_id());
        assert_eq!(second.attempt(), 2, "second pickup increments attempt");
    }

    #[sqlx::test]
    async fn record_success_moves_to_log(pool: PgPool) {
        let store = store(pool);
        let mut tx = store.pool().begin().await.unwrap();
        let _ = store
            .enqueue_tasks(&queue(), vec![make_input(Uuid::new_v4())], &mut tx)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let picked = store
            .pick_new_task(&queue(), Duration::seconds(300))
            .await
            .unwrap()
            .unwrap();

        let mut tx = store.pool().begin().await.unwrap();
        store
            .record_task_success(picked.id, Some("ok"), &mut tx)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let next = store
            .pick_new_task(&queue(), Duration::seconds(300))
            .await
            .unwrap();
        assert!(next.is_none(), "no further tasks after success");
    }

    #[sqlx::test]
    async fn record_failure_retries_then_archives(pool: PgPool) {
        let store = store(pool);
        let mut tx = store.pool().begin().await.unwrap();
        let _ = store
            .enqueue_tasks(&queue(), vec![make_input(Uuid::new_v4())], &mut tx)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        // Pick + retryable failure.
        let picked = store
            .pick_new_task(&queue(), Duration::seconds(300))
            .await
            .unwrap()
            .unwrap();
        let mut tx = store.pool().begin().await.unwrap();
        store
            .record_task_failure(picked.id, "transient", 3, &mut tx)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        // Task should still be schedulable.
        let retried = store
            .pick_new_task(&queue(), Duration::seconds(300))
            .await
            .unwrap()
            .expect("task should be back on the queue after retryable failure");
        assert_eq!(retried.task_id(), picked.task_id());
        assert_eq!(retried.attempt(), 2);

        // Terminal failure (attempt >= max_retries).
        let mut tx = store.pool().begin().await.unwrap();
        store
            .record_task_failure(retried.id, "terminal", 1, &mut tx)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let next = store
            .pick_new_task(&queue(), Duration::seconds(300))
            .await
            .unwrap();
        assert!(
            next.is_none(),
            "task should be archived after terminal failure"
        );
    }

    #[sqlx::test]
    async fn heartbeat_reports_state(pool: PgPool) {
        let store = store(pool);

        let mut tx = store.pool().begin().await.unwrap();
        let _ = store
            .enqueue_tasks(&queue(), vec![make_input(Uuid::new_v4())], &mut tx)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let picked = store
            .pick_new_task(&queue(), Duration::seconds(300))
            .await
            .unwrap()
            .unwrap();

        let mut tx = store.pool().begin().await.unwrap();
        let state = store
            .check_and_heartbeat_task(picked.id, 0.5, None, &mut tx)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        assert_eq!(state, TaskCheckState::Continue);
    }
}
