-- Background task queue tables, ported from lakekeeper/lakekeeper, Apache-2.0.
-- Adapted (collapsed) from the lakekeeper migrations:
--   * 20250523101407_tasks_store_their_state.sql
--   * 20250826171037_task_runtime_information.sql
--   * 20251228101923_enable_tasks_on_project_level.sql
-- Source: https://github.com/lakekeeper/lakekeeper/tree/05cce5797d3321ce30c59e034f19a3861c39cd88/crates/lakekeeper/migrations
--
-- Adjustments vs upstream:
--   * Unity Catalog has no Iceberg `project` or `warehouse` concepts. Those
--     columns are dropped; tasks are scoped on `(entity_type, entity_id)`.
--   * `entity_type` is stored as `text` rather than a closed PostgreSQL enum
--     so the framework stays generic and doesn't need a schema migration
--     every time a new UC resource kind is added.

CREATE TYPE task_intermediate_status AS ENUM ('scheduled', 'running', 'should-stop');
CREATE TYPE task_final_status AS ENUM ('failed', 'cancelled', 'success');

-- Active queue. One row per logical task; on pickup `attempt` is incremented.
CREATE TABLE task (
    task_id uuid PRIMARY KEY DEFAULT uuidv7(),
    queue_name text NOT NULL,
    status task_intermediate_status NOT NULL DEFAULT 'scheduled',
    parent_task_id uuid,
    scheduled_for timestamptz NOT NULL DEFAULT now(),
    picked_up_at timestamptz,
    last_heartbeat_at timestamptz,
    attempt integer NOT NULL DEFAULT 0,
    task_data jsonb NOT NULL,
    progress real NOT NULL DEFAULT 0.0,
    execution_details jsonb,
    -- `entity_type` is free-form text; `entity_id` / `entity_name` are NULL
    -- for system-level (catalog-wide) tasks.
    entity_type text,
    entity_id uuid,
    entity_name text[],
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz,
    -- Either both entity_id and entity_name are set (resource-scoped task)
    -- or both are NULL (system-level task). `entity_type` must be set when
    -- the task is resource-scoped.
    CONSTRAINT task_entity_check CHECK (
        (entity_type IS NULL AND entity_id IS NULL AND entity_name IS NULL)
        OR (entity_type IS NOT NULL AND entity_id IS NOT NULL AND entity_name IS NOT NULL)
    )
);

SELECT trigger_updated_at('task');

-- Deduplication index: one active task per (queue, entity) tuple. NULLS NOT
-- DISTINCT ensures system-level tasks (NULL entity) for the same queue are
-- also deduplicated.
CREATE UNIQUE INDEX task_entity_queue_name_idx
    ON task (entity_type, entity_id, queue_name)
    NULLS NOT DISTINCT;

-- Picker query covering index.
CREATE INDEX task_queue_name_status_scheduled_for_idx
    ON task (queue_name, status, scheduled_for);

CREATE INDEX task_entity_type_entity_id_created_at_idx
    ON task (entity_type, entity_id, created_at DESC);

-- Per-queue runtime configuration. Optional `max_time_since_last_heartbeat`
-- overrides the framework-provided default in `TaskConfig`.
CREATE TABLE task_config (
    task_config_id uuid PRIMARY KEY DEFAULT uuidv7(),
    queue_name text NOT NULL,
    config jsonb NOT NULL,
    max_time_since_last_heartbeat interval,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz,
    UNIQUE (queue_name)
);

SELECT trigger_updated_at('task_config');

-- Append-only audit log of completed/failed/cancelled task attempts.
CREATE TABLE task_log (
    task_id uuid NOT NULL,
    attempt integer NOT NULL,
    queue_name text NOT NULL,
    task_data jsonb NOT NULL,
    status task_final_status NOT NULL,
    message text,
    progress real NOT NULL DEFAULT 0.0,
    execution_details jsonb,
    parent_task_id uuid,
    entity_type text,
    entity_id uuid,
    entity_name text[],
    attempt_scheduled_for timestamptz NOT NULL,
    last_heartbeat_at timestamptz,
    started_at timestamptz,
    duration interval,
    task_created_at timestamptz NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (task_id, attempt)
);

CREATE INDEX task_log_queue_name_created_at_idx
    ON task_log (queue_name, created_at DESC);

CREATE INDEX task_log_entity_type_entity_id_task_created_at_idx
    ON task_log (entity_type, entity_id, task_created_at DESC);
