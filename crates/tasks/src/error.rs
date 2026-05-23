// Design ported from lakekeeper/lakekeeper, Apache-2.0.
// Upstream commit: https://github.com/lakekeeper/lakekeeper/tree/05cce5797d3321ce30c59e034f19a3861c39cd88
//
// Errors are reshaped to a thiserror-based enum to avoid pulling in
// `iceberg-ext`/`IcebergErrorResponse` from the upstream codebase.

use crate::types::{TaskAttemptId, TaskQueueName};

/// Result alias for fallible task-store and runtime operations.
pub type Result<T, E = TaskError> = std::result::Result<T, E>;

/// Errors that the task framework can surface.
///
/// Storage backends should map their native errors into one of the variants
/// below so workers and registry callers can handle them uniformly.
#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("failed to serialize {kind} for task queue `{queue}`: {source}")]
    Serialization {
        kind: &'static str,
        queue: TaskQueueName,
        #[source]
        source: serde_json::Error,
    },

    #[error("failed to deserialize {kind} for task queue `{queue}`: {source}")]
    Deserialization {
        kind: &'static str,
        queue: TaskQueueName,
        #[source]
        source: serde_json::Error,
    },

    #[error("task attempt {id} not found in active tasks")]
    TaskNotFound { id: TaskAttemptId },

    #[error("task attempt {id} has already been recorded as {outcome}")]
    AttemptAlreadyRecorded {
        id: TaskAttemptId,
        outcome: &'static str,
    },

    #[error("internal error: {0}")]
    Internal(String),
}

impl TaskError {
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}
