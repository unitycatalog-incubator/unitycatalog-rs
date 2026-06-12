//! Append a batch of rows to an existing Unity Catalog catalog-managed Delta table.
//!
//! Drives the kernel write `Transaction` through our [`UnityCatalogCommitter`]:
//!
//! 1. `loadTable` → the catalog's ratified commit tail + latest version.
//! 2. Build a kernel snapshot from that tail (catalog is source of truth), over a
//!    credentialed object store + engine.
//! 3. `snapshot.transaction(committer, engine)` → write parquet via
//!    `DefaultEngine::write_parquet` → `add_files` → `commit`. The committer stages the
//!    commit and ratifies it via UC `updateTable add-commit`.
//!
//! Publishing the staged commit to `_delta_log/<v>.json` is intentionally *not* done here:
//! UC's `loadTable` returns the ratified-but-unpublished commit in its tail, so readers
//! (via `build_catalog_managed_snapshot`) see it without publish. Publish is a maintenance
//! optimization left as a follow-up.

use std::sync::Arc;

use datafusion::arrow::array::RecordBatch;
use delta_kernel::engine::arrow_data::ArrowEngineData;
use delta_kernel::engine::default::DefaultEngineBuilder;
use delta_kernel::snapshot::Snapshot as KernelSnapshot;
use delta_kernel::transaction::CommitResult;
use delta_kernel::{LogPath, Version};
use unitycatalog_common::models::delta::v1::DeltaCommit;
use unitycatalog_object_store::{TableOperation, UnityObjectStoreFactory};
use url::Url;

use super::committer::UnityCatalogCommitter;
use super::create::CreateManagedTableError;

/// Append `batch` as a new commit to the existing managed table `catalog.schema.table`.
///
/// Returns the committed table version. `engine_info` is recorded in the commit's `commitInfo`.
pub async fn append_to_managed_table(
    factory: Arc<UnityObjectStoreFactory>,
    catalog: &str,
    schema: &str,
    table: &str,
    batch: RecordBatch,
    engine_info: &str,
) -> Result<Version, CreateManagedTableError> {
    let client = Arc::new(factory.unity_client().delta_v1());

    // 1. Ask the catalog for the table id, location, and ratified commit tail.
    let loaded = client.load_table(catalog, schema, table).await?;
    let table_id = loaded
        .metadata
        .properties
        .get("io.unitycatalog.tableId")
        .cloned()
        .ok_or_else(|| {
            CreateManagedTableError::other(format!(
                "table {catalog}.{schema}.{table} is missing the required \
                 io.unitycatalog.tableId property (ManagedTablesSpec §table properties)"
            ))
        })?;
    let location = Url::parse(&ensure_trailing_slash(&loaded.metadata.location))
        .map_err(|e| CreateManagedTableError::other(format!("invalid table location: {e}")))?;
    let commits = loaded.commits.as_deref().unwrap_or(&[]);
    let latest = loaded
        .latest_table_version
        .unwrap_or(loaded.metadata.last_commit_version.unwrap_or(0));
    let latest: Version = latest
        .try_into()
        .map_err(|_| CreateManagedTableError::other("negative latest_table_version"))?;

    // 2. Credentialed object store (table exists now, so vend by name) + kernel engine.
    let uc_store = factory
        .for_table(
            format!("{catalog}.{schema}.{table}"),
            TableOperation::ReadWrite,
        )
        .await?;
    let engine = DefaultEngineBuilder::new(uc_store.root()).build();

    // Build the kernel snapshot from the catalog's ratified state. A catalog-managed table
    // always requires `max_catalog_version` (even with an empty unbackfilled tail — e.g. a
    // freshly created table whose only commit is the published `0.json`).
    let snapshot = KernelSnapshot::builder_for(location.as_str())
        .with_log_tail(to_log_tail(&location, commits)?)
        .with_max_catalog_version(latest)
        .build(&engine)
        .map_err(CreateManagedTableError::Kernel)?;

    // 3. Write the batch through a transaction committed by our catalog committer.
    let committer = UnityCatalogCommitter::new(client, catalog, schema, table, table_id);
    let mut txn = snapshot
        .transaction(Box::new(committer), &engine)
        .map_err(CreateManagedTableError::Kernel)?
        .with_engine_info(engine_info);

    let write_context = txn
        .unpartitioned_write_context()
        .map_err(CreateManagedTableError::Kernel)?;
    let add_metadata = engine
        .write_parquet(&ArrowEngineData::new(batch), &write_context)
        .await
        .map_err(CreateManagedTableError::Kernel)?;
    txn.add_files(add_metadata);

    match txn
        .commit(&engine)
        .map_err(CreateManagedTableError::Kernel)?
    {
        CommitResult::CommittedTransaction(c) => Ok(c.commit_version()),
        CommitResult::ConflictedTransaction(c) => Err(CreateManagedTableError::other(format!(
            "commit conflicted at version {}",
            c.conflict_version()
        ))),
        CommitResult::RetryableTransaction(_) => Err(CreateManagedTableError::other(
            "retryable error during append commit",
        )),
    }
}

/// Map the loadTable commit tail to kernel staged-commit [`LogPath`]s (mirrors
/// `catalog::kernel::to_log_tail`).
fn to_log_tail(
    location: &Url,
    commits: &[DeltaCommit],
) -> Result<Vec<LogPath>, CreateManagedTableError> {
    let mut sorted: Vec<&DeltaCommit> = commits.iter().collect();
    sorted.sort_by_key(|c| c.version);
    sorted
        .into_iter()
        .map(|c| {
            let size: u64 = c
                .file_size
                .try_into()
                .map_err(|_| CreateManagedTableError::other("negative commit file_size"))?;
            LogPath::staged_commit(
                location.clone(),
                &c.file_name,
                c.file_modification_timestamp,
                size,
            )
            .map_err(CreateManagedTableError::Kernel)
        })
        .collect()
}

fn ensure_trailing_slash(s: &str) -> String {
    if s.ends_with('/') {
        s.to_string()
    } else {
        format!("{s}/")
    }
}
