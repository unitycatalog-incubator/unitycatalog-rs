// Design ported from lakekeeper/lakekeeper, Apache-2.0.
// Upstream source: https://github.com/lakekeeper/lakekeeper/blob/05cce5797d3321ce30c59e034f19a3861c39cd88/crates/lakekeeper/src/service/tasks/mod.rs
//
// Adapted for unitycatalog-rs:
//   * Dropped `WarehouseId` / `ProjectId` scoping — UC does not yet model them.
//   * Replaced `WarehouseTaskEntityId` (table/view enum) with a generic
//     `TaskEntity` carrying `entity_type` as a free-form string so the
//     framework stays UC-agnostic.
//   * Dropped utoipa / open-api derives.

use std::{fmt::Debug, ops::Deref};

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use uuid::Uuid;

/// Default maximum number of attempts a single task is allowed before being
/// marked permanently failed. Mirrors lakekeeper's default.
pub const DEFAULT_MAX_RETRIES: i32 = 5;

/// A queue identifier. Two queues with the same `TaskQueueName` share the same
/// configuration, history, and worker pool.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(transparent)]
pub struct TaskQueueName(String);

impl TaskQueueName {
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl Deref for TaskQueueName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: AsRef<str>> From<T> for TaskQueueName {
    fn from(name: T) -> Self {
        Self(name.as_ref().to_string())
    }
}

impl std::fmt::Display for TaskQueueName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Identifier for a logical task instance (independent of which attempt is
/// currently executing).
#[derive(Hash, Debug, Clone, PartialEq, Serialize, Deserialize, Copy, Eq)]
#[serde(transparent)]
pub struct TaskId(Uuid);

impl TaskId {
    pub fn new(id: Uuid) -> Self {
        Self(id)
    }

    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for TaskId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

impl From<TaskId> for Uuid {
    fn from(id: TaskId) -> Self {
        id.0
    }
}

impl Deref for TaskId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Identifies a specific attempt of a task. `attempt` increments each time the
/// task is picked up by a worker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskAttemptId {
    pub task_id: TaskId,
    pub attempt: i32,
}

impl TaskAttemptId {
    pub fn new(task_id: TaskId, attempt: i32) -> Self {
        Self { task_id, attempt }
    }
}

impl std::fmt::Display for TaskAttemptId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (attempt {})", self.task_id, self.attempt)
    }
}

impl AsRef<TaskAttemptId> for TaskAttemptId {
    fn as_ref(&self) -> &TaskAttemptId {
        self
    }
}

/// The thing a task acts on.
///
/// Mirrors lakekeeper's `TaskEntity` but stays generic: `entity_type` is a
/// free-form string instead of a closed enum so the framework does not need to
/// know about specific UC resource kinds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "kind")]
pub enum TaskEntity {
    /// A catalog-wide task with no associated resource (e.g. periodic
    /// housekeeping).
    System,
    /// A task scoped to a specific UC resource. `entity_type` mirrors UC
    /// resource kinds such as `"table"`, `"share"`, `"credential"`, etc.
    Resource {
        entity_type: String,
        entity_id: Uuid,
        entity_name: Vec<String>,
    },
}

impl TaskEntity {
    pub fn entity_type(&self) -> Option<&str> {
        match self {
            TaskEntity::System => None,
            TaskEntity::Resource { entity_type, .. } => Some(entity_type.as_str()),
        }
    }

    pub fn entity_id(&self) -> Option<Uuid> {
        match self {
            TaskEntity::System => None,
            TaskEntity::Resource { entity_id, .. } => Some(*entity_id),
        }
    }

    pub fn entity_name(&self) -> Option<&[String]> {
        match self {
            TaskEntity::System => None,
            TaskEntity::Resource { entity_name, .. } => Some(entity_name.as_slice()),
        }
    }
}

/// Persistent metadata for a task instance (read from the DB on pickup).
#[derive(Debug, Clone, PartialEq)]
pub struct TaskMetadata {
    pub parent_task_id: Option<TaskId>,
    pub scheduled_for: DateTime<Utc>,
    pub entity: TaskEntity,
}

/// Metadata supplied when scheduling a new task. `scheduled_for: None` defaults
/// to "now" on insert.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduleTaskMetadata {
    pub parent_task_id: Option<TaskId>,
    pub scheduled_for: Option<DateTime<Utc>>,
    pub entity: TaskEntity,
}

/// Raw input passed to `TaskStore::enqueue_tasks`. The payload has already been
/// serialised to a `Value` by the caller (typically `SpecializedTask`).
#[derive(Debug, Clone)]
pub struct TaskInput {
    pub task_metadata: ScheduleTaskMetadata,
    pub payload: serde_json::Value,
}

/// In-flight task as returned by `TaskStore::pick_new_task`.
#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    pub task_metadata: TaskMetadata,
    pub queue_name: TaskQueueName,
    pub id: TaskAttemptId,
    pub status: TaskIntermediateStatus,
    pub picked_up_at: Option<DateTime<Utc>>,
    pub config: Option<serde_json::Value>,
    pub data: serde_json::Value,
}

impl Task {
    pub fn task_id(&self) -> TaskId {
        self.id.task_id
    }

    pub fn attempt(&self) -> i32 {
        self.id.attempt
    }

    pub fn id(&self) -> TaskAttemptId {
        self.id
    }
}

impl AsRef<TaskAttemptId> for Task {
    fn as_ref(&self) -> &TaskAttemptId {
        &self.id
    }
}

/// Status of an active (non-terminal) task.
///
/// Stored in the Postgres `task` table as a `task_intermediate_status` enum.
#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "task_intermediate_status", rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum TaskIntermediateStatus {
    Scheduled,
    Running,
    ShouldStop,
}

/// Terminal task outcome, stored in `task_log.status`.
#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "task_final_status", rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum TaskOutcome {
    Failed,
    Cancelled,
    Success,
}

/// Result of `check_and_heartbeat_task`: tells the worker whether to keep
/// going, stop cleanly, or abort because the task is no longer active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[must_use]
pub enum TaskCheckState {
    /// External signal asked the worker to stop (e.g. via management API).
    Stop,
    /// Continue executing.
    Continue,
    /// The task is no longer active (e.g. cancelled, deleted, completed).
    NotActive,
}

impl TaskCheckState {
    pub fn should_terminate(&self) -> bool {
        matches!(self, TaskCheckState::Stop | TaskCheckState::NotActive)
    }

    pub fn should_report_termination(&self) -> bool {
        matches!(self, TaskCheckState::Stop)
    }
}

/// Internal status passed to `record_task_*` for logging/formatting.
#[derive(Debug, Clone)]
pub enum Status<'a> {
    Success(Option<&'a str>),
    Failure(&'a str, i32),
}

impl std::fmt::Display for Status<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Success(details) => write!(f, "success ({})", details.unwrap_or("")),
            Status::Failure(details, _) => write!(f, "failure ({details})"),
        }
    }
}

// -- typed-task traits ------------------------------------------------------

/// Per-queue static configuration. Mirrors lakekeeper's `TaskConfig`.
pub trait TaskConfig: Serialize + DeserializeOwned + Clone + Send + Sync + 'static {
    fn queue_name() -> &'static TaskQueueName;

    fn max_time_since_last_heartbeat() -> Duration;

    fn max_retries() -> i32 {
        DEFAULT_MAX_RETRIES
    }
}

/// Payload type for a queue. The framework only requires that it round-trips
/// through JSON.
pub trait TaskData: Clone + Serialize + DeserializeOwned + Send + Sync + 'static {}

/// Optional progress / execution-detail payload published via heartbeats.
pub trait TaskExecutionDetails:
    Clone + Serialize + DeserializeOwned + Send + Sync + 'static
{
}
