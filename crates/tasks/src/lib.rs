//! Postgres-backed background task queue framework.
//!
//! See the crate-level `README.md` for design notes, scope, and attribution.
//! The architecture is ported from
//! [lakekeeper/lakekeeper](https://github.com/lakekeeper/lakekeeper)'s
//! `service/tasks/` module (Apache-2.0).

pub mod error;
mod registry;
mod runner;
mod specialized;
mod store;
mod types;

pub use error::{Result, TaskError};
pub use registry::{
    QueueApiConfig, QueueRegistration, RegisteredTaskQueues, TaskQueueRegistry, ValidatorFn,
};
pub use runner::{TaskQueueWorkerFn, TaskQueuesRunner};
pub use specialized::SpecializedTask;
pub use store::TaskStore;
pub use tokio_util::sync::CancellationToken;
pub use types::{
    DEFAULT_MAX_RETRIES, ScheduleTaskMetadata, Status, Task, TaskAttemptId, TaskCheckState,
    TaskConfig, TaskData, TaskEntity, TaskExecutionDetails, TaskId, TaskInput,
    TaskIntermediateStatus, TaskMetadata, TaskOutcome, TaskQueueName,
};
