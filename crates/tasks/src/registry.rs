// Design ported from lakekeeper/lakekeeper, Apache-2.0.
// Upstream source: https://github.com/lakekeeper/lakekeeper/blob/05cce5797d3321ce30c59e034f19a3861c39cd88/crates/lakekeeper/src/service/tasks/task_registry.rs
//
// Adapted for unitycatalog-rs:
//   * Dropped `QueueScope`, OpenAPI / utoipa integration, and the
//     warehouse/project distinctions — none of those concepts exist in UC.
//   * `RegisteredTaskQueues` still exposes a shared, interior-mutable view of
//     the API config so it can be embedded in axum app state for a future
//     management API. For now it only carries the validator function so
//     downstream code can validate user-supplied queue config payloads.

use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

use crate::runner::{QueueWorkerConfig, TaskQueueWorkerFn, TaskQueuesRunner};
use crate::types::{TaskConfig, TaskQueueName};

/// Validates a `serde_json::Value` against a queue's config schema.
pub type ValidatorFn =
    Arc<dyn Fn(serde_json::Value) -> serde_json::Result<()> + Send + Sync + 'static>;

#[derive(Clone)]
struct RegisteredQueue {
    api_config: QueueApiConfig,
    schema_validator_fn: ValidatorFn,
}

impl std::fmt::Debug for RegisteredQueue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegisteredQueue")
            .field("api_config", &self.api_config)
            .field("schema_validator_fn", &"Fn(...)")
            .finish()
    }
}

#[derive(Clone)]
struct RegisteredTaskQueueWorker {
    worker_fn: TaskQueueWorkerFn,
    num_workers: usize,
}

impl std::fmt::Debug for RegisteredTaskQueueWorker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegisteredTaskQueueWorker")
            .field("worker_fn", &"Fn(...)")
            .field("num_workers", &self.num_workers)
            .finish()
    }
}

/// Per-queue API metadata. Kept intentionally minimal here; a future
/// management API can attach more (e.g. OpenAPI schemas) without affecting the
/// runtime path.
#[derive(Clone, Debug)]
pub struct QueueApiConfig {
    pub queue_name: &'static TaskQueueName,
}

/// Description of a queue to register with the [`TaskQueueRegistry`].
#[derive(Clone)]
pub struct QueueRegistration {
    /// Static name of the queue.
    pub queue_name: &'static TaskQueueName,
    /// Worker loop. Receives a cancellation token and is expected to honour it.
    pub worker_fn: TaskQueueWorkerFn,
    /// Number of worker tasks to spawn locally for this queue.
    pub num_workers: usize,
}

impl std::fmt::Debug for QueueRegistration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueueRegistration")
            .field("queue_name", &self.queue_name)
            .field("worker_fn", &"Fn(...)")
            .field("num_workers", &self.num_workers)
            .finish()
    }
}

/// Container for registered queues that can be cloned into application state.
///
/// The interior state is shared with the parent [`TaskQueueRegistry`], so
/// registrations made after this view was constructed are visible to it.
#[derive(Clone, Default, Debug)]
pub struct RegisteredTaskQueues {
    queues: Arc<RwLock<HashMap<&'static TaskQueueName, RegisteredQueue>>>,
}

impl RegisteredTaskQueues {
    /// Returns the API config for every registered queue, sorted by name.
    pub async fn api_config(&self) -> Vec<QueueApiConfig> {
        let mut configs: Vec<_> = self
            .queues
            .read()
            .await
            .values()
            .map(|q| q.api_config.clone())
            .collect();
        configs.sort_by_key(|c| c.queue_name);
        configs
    }

    /// Returns the validator function for a given queue, or `None` if no queue
    /// with that name is registered.
    pub async fn validate_config_fn(&self, queue: &TaskQueueName) -> Option<ValidatorFn> {
        self.queues
            .read()
            .await
            .get(queue)
            .map(|q| Arc::clone(&q.schema_validator_fn))
    }

    /// Convenience helper to list queue names in deterministic order.
    pub async fn queue_names(&self) -> Vec<&'static TaskQueueName> {
        let mut v: Vec<_> = self.queues.read().await.keys().copied().collect();
        v.sort_unstable();
        v
    }
}

/// Registry for task queues and their worker pools.
#[derive(Debug, Clone)]
pub struct TaskQueueRegistry {
    registered_queues: Arc<RwLock<HashMap<&'static TaskQueueName, RegisteredQueue>>>,
    task_workers: Arc<RwLock<HashMap<&'static TaskQueueName, RegisteredTaskQueueWorker>>>,
}

impl Default for TaskQueueRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskQueueRegistry {
    pub fn new() -> Self {
        Self {
            registered_queues: Arc::new(RwLock::new(HashMap::new())),
            task_workers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a queue with its worker function and configuration type.
    ///
    /// Registering a queue with the same name twice logs a warning and
    /// overwrites the previous registration.
    pub async fn register_queue<T: TaskConfig>(&self, task_queue: QueueRegistration) -> &Self {
        let QueueRegistration {
            queue_name,
            worker_fn,
            num_workers,
        } = task_queue;

        let schema_validator_fn: ValidatorFn =
            Arc::new(|v| serde_json::from_value::<T>(v).map(|_| ()));
        let api_config = QueueApiConfig { queue_name };

        if self
            .registered_queues
            .write()
            .await
            .insert(
                queue_name,
                RegisteredQueue {
                    api_config,
                    schema_validator_fn,
                },
            )
            .is_some()
        {
            tracing::warn!("Overwriting registration for queue `{queue_name}`");
        }

        self.task_workers.write().await.insert(
            queue_name,
            RegisteredTaskQueueWorker {
                worker_fn,
                num_workers,
            },
        );
        self
    }

    /// Create a [`RegisteredTaskQueues`] view sharing state with this
    /// registry.
    pub fn registered_task_queues(&self) -> RegisteredTaskQueues {
        RegisteredTaskQueues {
            queues: self.registered_queues.clone(),
        }
    }

    /// Number of registered queues.
    pub async fn len(&self) -> usize {
        self.registered_queues.read().await.len()
    }

    pub async fn is_empty(&self) -> bool {
        self.registered_queues.read().await.is_empty()
    }

    /// Build a [`TaskQueuesRunner`] snapshot.
    ///
    /// Workers registered after `task_queues_runner` returns are **not**
    /// reflected in that runner — call this once all queues are registered.
    pub async fn task_queues_runner(
        &self,
        cancellation_token: CancellationToken,
    ) -> TaskQueuesRunner {
        let mut registered_task_queues = HashMap::new();

        let queues = self.registered_queues.read().await;
        let workers = self.task_workers.read().await;

        for name in queues.keys() {
            if let Some(worker) = workers.get(name) {
                registered_task_queues.insert(
                    *name,
                    QueueWorkerConfig {
                        worker_fn: Arc::clone(&worker.worker_fn),
                        num_workers: worker.num_workers,
                    },
                );
            }
        }

        TaskQueuesRunner {
            registered_queues: Arc::new(registered_task_queues),
            cancellation_token,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use serde::{Deserialize, Serialize};

    use super::*;

    #[derive(Clone, Debug, Serialize, Deserialize)]
    struct FirstConfig {
        test_field: String,
    }

    static FIRST_QUEUE_NAME: LazyLock<TaskQueueName> = LazyLock::new(|| "test-queue".into());

    impl TaskConfig for FirstConfig {
        fn queue_name() -> &'static TaskQueueName {
            &FIRST_QUEUE_NAME
        }

        fn max_time_since_last_heartbeat() -> chrono::Duration {
            chrono::Duration::seconds(300)
        }
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    struct SecondConfig {
        other_field: i32,
    }

    static SECOND_QUEUE_NAME: LazyLock<TaskQueueName> =
        LazyLock::new(|| "second-test-queue".into());

    impl TaskConfig for SecondConfig {
        fn queue_name() -> &'static TaskQueueName {
            &SECOND_QUEUE_NAME
        }

        fn max_time_since_last_heartbeat() -> chrono::Duration {
            chrono::Duration::seconds(300)
        }
    }

    fn noop_worker() -> TaskQueueWorkerFn {
        Arc::new(|_token| Box::pin(async {}))
    }

    #[tokio::test]
    async fn registry_starts_empty() {
        let registry = TaskQueueRegistry::new();
        assert_eq!(registry.len().await, 0);
        assert!(registry.is_empty().await);
        assert!(
            registry
                .registered_task_queues()
                .api_config()
                .await
                .is_empty()
        );
    }

    #[tokio::test]
    async fn registry_register_lookup() {
        let registry = TaskQueueRegistry::new();
        let view = registry.registered_task_queues();

        registry
            .register_queue::<FirstConfig>(QueueRegistration {
                queue_name: &FIRST_QUEUE_NAME,
                worker_fn: noop_worker(),
                num_workers: 1,
            })
            .await;

        assert_eq!(registry.len().await, 1);

        let api_config = view.api_config().await;
        assert_eq!(api_config.len(), 1);
        assert_eq!(api_config[0].queue_name, &*FIRST_QUEUE_NAME);

        assert!(view.validate_config_fn(&FIRST_QUEUE_NAME).await.is_some());
        assert!(
            view.validate_config_fn(&TaskQueueName::from("does-not-exist"))
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn registered_task_queues_share_state() {
        let registry = TaskQueueRegistry::new();
        let initial = registry.registered_task_queues();

        registry
            .register_queue::<FirstConfig>(QueueRegistration {
                queue_name: &FIRST_QUEUE_NAME,
                worker_fn: noop_worker(),
                num_workers: 1,
            })
            .await;

        let later = registry.registered_task_queues();
        registry
            .register_queue::<SecondConfig>(QueueRegistration {
                queue_name: &SECOND_QUEUE_NAME,
                worker_fn: noop_worker(),
                num_workers: 2,
            })
            .await;

        let initial_names = initial.queue_names().await;
        let later_names = later.queue_names().await;
        assert_eq!(initial_names.len(), 2);
        assert_eq!(initial_names, later_names);
    }

    #[tokio::test]
    async fn validator_accepts_well_formed_payload() {
        let registry = TaskQueueRegistry::new();
        registry
            .register_queue::<FirstConfig>(QueueRegistration {
                queue_name: &FIRST_QUEUE_NAME,
                worker_fn: noop_worker(),
                num_workers: 1,
            })
            .await;

        let view = registry.registered_task_queues();
        let validator = view
            .validate_config_fn(&FIRST_QUEUE_NAME)
            .await
            .expect("validator registered");

        assert!(validator(serde_json::json!({ "test_field": "hello" })).is_ok());
        assert!(validator(serde_json::json!({ "wrong": 1 })).is_err());
    }
}
