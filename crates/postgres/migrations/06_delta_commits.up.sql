-- Ratified-but-unpublished Delta catalog-managed commits.
--
-- Mirrors the Unity Catalog OSS `uc_delta_commits` table. The unique constraint
-- on (table_id, commit_version) is the real first-writer-wins arbiter: two
-- writers racing the same version, exactly one insert succeeds.
create table delta_commits (
    id uuid primary key default uuidv7(),
    table_id uuid not null,
    commit_version bigint not null,
    commit_filename text not null,
    commit_filesize bigint not null,
    commit_file_modification_timestamp timestamptz not null,
    -- The in-commit timestamp supplied by the Delta client.
    commit_timestamp timestamptz not null,
    -- Set on the highest commit once it has been fully backfilled: the row is
    -- retained as a version marker but hidden from `get_commits`.
    is_backfilled_latest boolean not null default false,
    created_at timestamptz not null default now(),
    updated_at timestamptz,
    constraint unique_delta_commit unique (table_id, commit_version)
);
select trigger_updated_at('delta_commits');
create index delta_commits_table_index on delta_commits (table_id, commit_version);
