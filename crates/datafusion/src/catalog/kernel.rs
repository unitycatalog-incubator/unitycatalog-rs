//! Catalog-managed snapshot construction.
//!
//! Bridges the `/delta/v1` loadTable response (catalog-ratified commit tail +
//! latest version) to a deltalake [`Snapshot`] for a catalog-managed
//! (coordinated-commit) Delta table.
//!
//! The catalog — not the filesystem — is the source of truth for a managed
//! table's latest version: the newest commits are ratified but may not yet be
//! backfilled into `_delta_log/` (they live as staged commits under
//! `_delta_log/_staged_commits/`). We build a `delta_kernel` snapshot from the
//! supplied commit tail + catalog version and wrap it with [`Snapshot::new`],
//! mirroring the reference `delta-kernel-unity-catalog` read path.

use datafusion::common::DataFusionError;
use delta_kernel::snapshot::Snapshot as KernelSnapshot;
use delta_kernel::{Engine, LogPath, Version};
use deltalake_core::DeltaTableConfig;
use deltalake_core::kernel::Snapshot;
use unitycatalog_common::models::delta::v1::DeltaCommit;
use url::Url;

use super::builder::TableProviderError;

/// Build a catalog-managed [`Snapshot`] from a table location, the catalog-ratified
/// commit tail, and the latest ratified version.
///
/// `commits` is the unbackfilled CCv2 tail from the loadTable response; it is
/// sorted ascending here (the kernel requires a contiguous, ascending log tail).
/// `latest_table_version` is the highest version the catalog has ratified, used as
/// the catalog-version cap and — when `at_version` is `None` — as the effective
/// target version.
pub(crate) fn build_catalog_managed_snapshot(
    engine: &dyn Engine,
    location: &Url,
    commits: &[DeltaCommit],
    latest_table_version: i64,
    at_version: Option<Version>,
) -> Result<Snapshot, TableProviderError> {
    let latest_table_version: Version = latest_table_version.try_into().map_err(|_| {
        DataFusionError::Plan(format!(
            "negative latest-table-version {latest_table_version} for '{location}'"
        ))
    })?;

    // The kernel joins log paths onto the table root via `Url::join`, which drops the
    // final segment unless the root ends in a slash; guarantee the trailing slash.
    let mut table_root = location.clone();
    if !table_root.path().ends_with('/') {
        table_root.set_path(&format!("{}/", table_root.path()));
    }

    let log_tail = to_log_tail(&table_root, commits)?;

    let mut builder = KernelSnapshot::builder_for(table_root)
        .with_max_catalog_version(latest_table_version)
        .with_log_tail(log_tail);
    if let Some(version) = at_version {
        builder = builder.at_version(version);
    }

    let inner = builder
        .build(engine)
        .map_err(|e| DataFusionError::External(Box::new(e)))?;

    Ok(Snapshot::new(inner, DeltaTableConfig::default()))
}

/// Map the loadTable commit tail to kernel staged-commit [`LogPath`]s, sorted
/// ascending by version (the kernel rejects a non-contiguous / unsorted tail).
/// Rejects a negative `file-size`, which can't be a valid byte count.
fn to_log_tail(
    table_root: &Url,
    commits: &[DeltaCommit],
) -> Result<Vec<LogPath>, TableProviderError> {
    let mut sorted: Vec<&DeltaCommit> = commits.iter().collect();
    sorted.sort_by_key(|c| c.version);

    sorted
        .into_iter()
        .map(|c| {
            let size: u64 = c.file_size.try_into().map_err(|_| {
                DataFusionError::Plan(format!(
                    "negative file-size {} for commit {} of '{table_root}'",
                    c.file_size, c.version
                ))
            })?;
            LogPath::staged_commit(
                table_root.clone(),
                &c.file_name,
                c.file_modification_timestamp,
                size,
            )
            .map_err(|e| DataFusionError::External(Box::new(e)))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn commit(version: i64, file_name: &str, file_size: i64) -> DeltaCommit {
        DeltaCommit {
            version,
            timestamp: 0,
            file_name: file_name.to_string(),
            file_size,
            file_modification_timestamp: 0,
        }
    }

    /// A valid staged-commit file name: `<20-digit version>.<uuid>.json`, matching
    /// what the catalog returns and what the kernel's log-path parser accepts.
    fn staged_name(version: u64) -> String {
        format!("{version:020}.3a0d65cd-4056-49b8-937b-95f9e3ee90e5.json")
    }

    #[test]
    fn maps_commit_tail_to_staged_log_paths() {
        let root = Url::parse("s3://bucket/table/").unwrap();
        // Deliberately out of order; sorting must yield a contiguous ascending tail
        // the kernel will accept. `LogPath::staged_commit` succeeding for each entry
        // also asserts our staged-commit file-name handling is valid.
        let commits = vec![
            commit(2, &staged_name(2), 20),
            commit(1, &staged_name(1), 10),
            commit(3, &staged_name(3), 30),
        ];

        let tail = to_log_tail(&root, &commits).unwrap();
        assert_eq!(tail.len(), 3);
    }

    #[test]
    fn rejects_negative_file_size() {
        let root = Url::parse("s3://bucket/table/").unwrap();
        let commits = vec![commit(1, &staged_name(1), -1)];

        let err = to_log_tail(&root, &commits).unwrap_err();
        assert!(
            err.to_string().contains("negative file-size"),
            "unexpected error: {err}"
        );
    }
}
