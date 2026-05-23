# unitycatalog-tasks

A Postgres-backed background task queue framework for the
[`unitycatalog-rs`](https://github.com/unitycatalog-incubator/unitycatalog-rs)
server. Workers run in-process alongside the HTTP server; tasks are persisted
in Postgres and picked up via `FOR UPDATE SKIP LOCKED`. Designed to host
catalog maintenance jobs (deferred deletes, log cleanup, credential rotation,
table snapshot maintenance) — **not** user-defined data pipelines.

## Acknowledgements

The architecture, table schema, and trait shapes in this crate are **ported
directly from
[lakekeeper/lakekeeper](https://github.com/lakekeeper/lakekeeper)'s
[`crates/lakekeeper/src/service/tasks/`](https://github.com/lakekeeper/lakekeeper/blob/main/crates/lakekeeper/src/service/tasks/)
module** (Apache-2.0). Specifically, we ported from upstream commit
[`05cce57`](https://github.com/lakekeeper/lakekeeper/commit/05cce5797d3321ce30c59e034f19a3861c39cd88).

Huge thanks to the lakekeeper maintainers — in particular the registry/runner
split and `SpecializedTask` design from
[issue #1310](https://github.com/lakekeeper/lakekeeper/issues/1310). Each ported
source file carries a top-of-file comment pointing back at the corresponding
upstream file.

## Why a separate crate instead of depending on `lakekeeper` directly

We considered consuming the upstream queue as a dependency and decided against
it. Three blockers:

1. **Not on crates.io.** None of lakekeeper's workspace crates
   (`lakekeeper`, `lakekeeper-bin`, `iceberg-ext`, `authz-openfga`,
   `lakekeeper-io`) are published. Releases ship as Docker images and binaries.
2. **Task queue is not a standalone crate.** It lives inside the monolithic
   `lakekeeper` crate at `crates/lakekeeper/src/service/tasks/` and is not
   feature-isolated.
3. **Heavy transitive dependencies.** Even with `default-features = false`,
   pulling in `lakekeeper` brings `iceberg` (their fork), `iceberg-ext`,
   `lakekeeper-io`, AWS / Azure / GCS SDKs, `cloudevents-sdk`,
   `axum-prometheus`, `figment`, `limes`, `moka`, `vaultrs`, `quick-xml`, and
   ~60+ other crates. The queue module is also tightly coupled to internal
   lakekeeper types (`CatalogStore`, `RequestMetadata`, `iceberg::TableIdent`,
   the global `CONFIG`).

Maintaining a small, focused copy here lets us depend on roughly ten crates
(`sqlx`, `serde`, `chrono`, `uuid`, `tokio`, `tokio-util`, `tracing`,
`async-trait`, `thiserror`, `fastrand`, `futures`) and to evolve the API for
Unity Catalog semantics.

## API compatibility goal

The public type and trait names mirror lakekeeper's surface so that if either
project later extracts `service/tasks/` into a standalone `lakekeeper-tasks`
crate, the swap is mechanical:

| `unitycatalog-tasks`                  | lakekeeper equivalent                |
| ------------------------------------- | ------------------------------------ |
| `TaskQueueRegistry`                   | `TaskQueueRegistry`                  |
| `TaskQueuesRunner`                    | `TaskQueuesRunner`                   |
| `TaskQueueWorkerFn`                   | `TaskQueueWorkerFn`                  |
| `SpecializedTask<Q, D, E>`            | `SpecializedTask<Q, D, E>`           |
| `TaskConfig` / `TaskData` / ...       | `TaskConfig` / `TaskData` / ...      |
| `TaskStore` trait                     | `CatalogTaskOps` trait               |
| `TaskCheckState`, `TaskAttemptId`, …  | identical names                      |

## Scope

This crate is *infrastructure only*. It provides:

- Type-safe per-queue task definitions (`TaskConfig`, `TaskData`,
  `TaskExecutionDetails`, `SpecializedTask<Q, D, E>`).
- A `TaskQueueRegistry` for registering queues and their worker functions.
- A `TaskQueuesRunner` that spawns and supervises the worker tasks (auto-restart
  on crash, cooperative shutdown via `CancellationToken`).
- A `TaskStore` trait describing the storage primitives (`pick_new_task`,
  `enqueue_tasks`, `check_and_heartbeat_task`, `record_task_success`,
  `record_task_failure`).

It does **not** ship any specific worker implementations, a management HTTP
API, or an in-memory backend. Those are intentionally left to follow-up work
once we have concrete catalog jobs to run.

## What this crate is not

`unitycatalog-tasks` is for **system-initiated, transactionally-enqueued
maintenance work** — the catalog managing its own state. It is not a
substitute for a data-pipeline orchestrator like
[rivers](https://github.com/ion-elgreco/rivers), Dagster, Airflow, or Kestra.
Those tools belong on the *consumer* side of Unity Catalog: they read from and
write to UC via its REST API.

## License

Apache-2.0 — matches upstream lakekeeper and the rest of this repository.
