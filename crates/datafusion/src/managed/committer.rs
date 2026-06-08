//! A Unity Catalog catalog-managed [`Committer`] backed by our [`DeltaV1Client`].
//!
//! Ported from the buoyant kernel fork's experimental `delta-kernel-unity-catalog`
//! `UCCommitter` (`delta-kernel-unity-catalog/src/committer.rs`), but bound directly to
//! the unitycatalog-rs `DeltaV1Client` (`/delta/v1` `updateTable` `add-commit`) instead of
//! a separate `CommitClient` trait — so unitycatalog-rs stays the single UC client. Keep
//! this close to the upstream source so re-syncing (or depending on the crate once it
//! stabilizes) stays mechanical.
//!
//! For version 0 (table creation) the committer writes `00…0.json` directly to the
//! published commit path; the connector finalizes the table in UC via `createTable`
//! afterward (see [`crate::managed::create_managed_table`]). For version >= 1 it writes a
//! staged commit and calls the UC `add-commit` API to ratify it.
//!
//! NOTE: [`Committer::commit`] is synchronous but `DeltaV1Client` is async, so we bridge
//! via `tokio::task::block_in_place` + `Handle::block_on` — this committer therefore
//! requires a multi-threaded tokio runtime (the default engine's runtime is fine).

use std::sync::Arc;

use delta_kernel::committer::{
    CommitMetadata, CommitResponse, CommitType, Committer, PublishMetadata,
};
use delta_kernel::{DeltaResult, DeltaResultIterator, Engine, Error as DeltaError, FilteredEngineData};
use tracing::{debug, info};
use unitycatalog_client::DeltaV1Client;
use unitycatalog_common::models::delta::v1::{
    DeltaCommit, DeltaTableRequirement, DeltaTableUpdate, DeltaUpdateTableRequest,
};

// UC catalog-managed contract identifiers (mirror the fork's `constants`).
const CATALOG_MANAGED_FEATURE: &str = "catalogManaged";
const VACUUM_PROTOCOL_CHECK_FEATURE: &str = "vacuumProtocolCheck";
const IN_COMMIT_TIMESTAMP_FEATURE: &str = "inCommitTimestamp";
const ENABLE_IN_COMMIT_TIMESTAMPS: &str = "delta.enableInCommitTimestamps";
const UC_TABLE_ID_KEY: &str = "io.unitycatalog.tableId";
const CLUSTERING_DOMAIN_NAME: &str = "delta.clustering";

macro_rules! require {
    ($cond:expr, $msg:expr) => {
        if !($cond) {
            return Err(DeltaError::generic($msg));
        }
    };
}

/// A Unity Catalog catalog-managed [`Committer`] for one table, addressed by its
/// three-level name (the key our `/delta/v1` `updateTable` endpoint uses) plus its UC
/// `table_id` (validated against the commit's `io.unitycatalog.tableId`).
#[derive(Clone)]
pub struct UnityCatalogCommitter {
    client: Arc<DeltaV1Client>,
    catalog: String,
    schema: String,
    table: String,
    table_id: String,
}

impl std::fmt::Debug for UnityCatalogCommitter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnityCatalogCommitter")
            .field("table", &format_args!("{}.{}.{}", self.catalog, self.schema, self.table))
            .field("table_id", &self.table_id)
            .finish()
    }
}

impl UnityCatalogCommitter {
    /// Create a committer for the table `catalog.schema.table` with UC id `table_id`.
    pub fn new(
        client: Arc<DeltaV1Client>,
        catalog: impl Into<String>,
        schema: impl Into<String>,
        table: impl Into<String>,
        table_id: impl Into<String>,
    ) -> Self {
        Self {
            client,
            catalog: catalog.into(),
            schema: schema.into(),
            table: table.into(),
            table_id: table_id.into(),
        }
    }

    fn has_catalog_managed_feature(commit_metadata: &CommitMetadata) -> bool {
        commit_metadata.has_writer_feature(CATALOG_MANAGED_FEATURE)
            && commit_metadata.has_reader_feature(CATALOG_MANAGED_FEATURE)
    }

    /// Validate protocol features + metadata properties for a UC catalog-managed table.
    fn validate_catalog_managed_state(&self, commit_metadata: &CommitMetadata) -> DeltaResult<()> {
        require!(
            commit_metadata.commit_type() != CommitType::UpgradeToCatalogManaged,
            "upgrade to catalog-managed is not supported"
        );
        require!(
            commit_metadata.commit_type() != CommitType::DowngradeToPathBased,
            "downgrade to path-based is not supported"
        );
        require!(
            Self::has_catalog_managed_feature(commit_metadata),
            "table is missing required feature: catalogManaged"
        );
        require!(
            commit_metadata.has_writer_feature(VACUUM_PROTOCOL_CHECK_FEATURE)
                && commit_metadata.has_reader_feature(VACUUM_PROTOCOL_CHECK_FEATURE),
            "table is missing required feature: vacuumProtocolCheck"
        );
        require!(
            commit_metadata.has_writer_feature(IN_COMMIT_TIMESTAMP_FEATURE),
            "table is missing required feature: inCommitTimestamp"
        );

        let config = commit_metadata
            .metadata_configuration()
            .ok_or_else(|| DeltaError::generic("commit metadata is missing table configuration"))?;
        let table_id = config
            .get(UC_TABLE_ID_KEY)
            .ok_or_else(|| DeltaError::generic(format!("table is missing property: {UC_TABLE_ID_KEY}")))?;
        require!(
            table_id == &self.table_id,
            format!("table ID mismatch: committer={}, metadata={table_id}", self.table_id)
        );
        require!(
            config.get(ENABLE_IN_COMMIT_TIMESTAMPS).map(String::as_str) == Some("true"),
            "in-commit timestamps must be enabled (delta.enableInCommitTimestamps=true)"
        );
        Ok(())
    }

    /// Reject ALTER TABLE changes (protocol / metadata / clustering) on a data commit —
    /// v1 supports append-style commits only, matching the upstream committer.
    fn validate_no_alter_table_changes(commit_metadata: &CommitMetadata) -> DeltaResult<()> {
        require!(
            !commit_metadata.has_protocol_change(),
            "changing table protocol is not supported in a catalog-managed commit"
        );
        require!(
            !commit_metadata.has_metadata_change(),
            "changing table metadata is not supported in a catalog-managed commit"
        );
        require!(
            !commit_metadata.has_domain_metadata_change(CLUSTERING_DOMAIN_NAME),
            "changing clustering columns is not supported in a catalog-managed commit"
        );
        Ok(())
    }

    /// Version 0 (create): validate the contract, then write `00…0.json` directly to the
    /// published path. Conflict if it already exists.
    fn commit_version_0(
        &self,
        engine: &dyn Engine,
        actions: DeltaResultIterator<'_, FilteredEngineData>,
        commit_metadata: &CommitMetadata,
    ) -> DeltaResult<CommitResponse> {
        self.validate_catalog_managed_state(commit_metadata)?;
        let published = commit_metadata.published_commit_path()?;
        match engine
            .json_handler()
            .write_json_file(&published, Box::new(actions), false)
        {
            Ok(()) => {
                info!("wrote version 0 commit for UC managed table creation");
                let file_meta = engine.storage_handler().head(&published)?;
                Ok(CommitResponse::Committed { file_meta })
            }
            Err(DeltaError::FileAlreadyExists(_)) => Ok(CommitResponse::Conflict { version: 0 }),
            Err(e) => Err(e),
        }
    }

    /// Version >= 1: write a staged commit, then `updateTable add-commit` to ratify it.
    fn commit_version_non_zero(
        &self,
        engine: &dyn Engine,
        actions: DeltaResultIterator<'_, FilteredEngineData>,
        commit_metadata: CommitMetadata,
    ) -> DeltaResult<CommitResponse> {
        self.validate_catalog_managed_state(&commit_metadata)?;
        Self::validate_no_alter_table_changes(&commit_metadata)?;

        let staged = commit_metadata.staged_commit_path()?;
        engine
            .json_handler()
            .write_json_file(&staged, Box::new(actions), false)?;
        let committed = engine.storage_handler().head(&staged)?;
        debug!(path = %staged, "wrote staged commit file");

        let version: i64 = commit_metadata
            .version()
            .try_into()
            .map_err(|_| DeltaError::generic("commit version does not fit into i64 for UC commit"))?;
        let file_name = staged
            .path_segments()
            .and_then(|mut s| s.next_back())
            .ok_or_else(|| DeltaError::generic("staged commit path had no file name"))?
            .to_string();
        let file_size: i64 = committed
            .size
            .try_into()
            .map_err(|_| DeltaError::generic("staged commit size does not fit into i64"))?;

        let request = DeltaUpdateTableRequest {
            requirements: vec![DeltaTableRequirement::AssertTableUuid {
                uuid: self.table_id.clone(),
            }],
            updates: vec![DeltaTableUpdate::AddCommit {
                commit: DeltaCommit {
                    version,
                    timestamp: commit_metadata.in_commit_timestamp(),
                    file_name,
                    file_size,
                    file_modification_timestamp: committed.last_modified,
                },
                uniform: None,
            }],
        };

        match self.block_on_update(request) {
            Ok(()) => Ok(CommitResponse::Committed { file_meta: committed }),
            // A version conflict (UC returns 409) becomes a kernel Conflict so the caller's
            // commit loop can rebuild and retry, rather than a hard error.
            Err(e) if is_conflict(&e) => Ok(CommitResponse::Conflict {
                version: commit_metadata.version(),
            }),
            Err(e) => Err(DeltaError::generic(format!("UC add-commit failed: {e}"))),
        }
    }

    /// Run an async `update_table` to completion from the synchronous `Committer::commit`.
    /// Requires a multi-threaded tokio runtime (matches the upstream committer).
    fn block_on_update(
        &self,
        request: DeltaUpdateTableRequest,
    ) -> unitycatalog_client::Result<()> {
        let handle = tokio::runtime::Handle::try_current().map_err(|_| {
            unitycatalog_client::Error::Generic(
                "UnityCatalogCommitter must be used within a tokio runtime".to_string(),
            )
        })?;
        let client = self.client.clone();
        let (catalog, schema, table) = (self.catalog.clone(), self.schema.clone(), self.table.clone());
        tokio::task::block_in_place(|| {
            handle.block_on(async move {
                client
                    .update_table(&catalog, &schema, &table, &request)
                    .await
                    .map(|_| ())
            })
        })
    }
}

impl Committer for UnityCatalogCommitter {
    fn commit(
        &self,
        engine: &dyn Engine,
        actions: DeltaResultIterator<'_, FilteredEngineData>,
        commit_metadata: CommitMetadata,
    ) -> DeltaResult<CommitResponse> {
        if commit_metadata.version() == 0 {
            return self.commit_version_0(engine, actions, &commit_metadata);
        }
        self.commit_version_non_zero(engine, actions, commit_metadata)
    }

    fn is_catalog_committer(&self) -> bool {
        true
    }

    fn publish(&self, engine: &dyn Engine, publish_metadata: PublishMetadata) -> DeltaResult<()> {
        // Publish = copy each ratified staged commit to its published `_delta_log/<v>.json`
        // path (atomic / put-if-absent). An already-published version is not an error.
        for catalog_commit in publish_metadata.commits_to_publish() {
            match engine
                .storage_handler()
                .copy_atomic(catalog_commit.location(), catalog_commit.published_location())
            {
                Ok(()) | Err(DeltaError::FileAlreadyExists(_)) => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

/// Whether a client error is a UC version conflict (HTTP 409). The server returns 409 for
/// a commit-version conflict — either as a typed `AlreadyExists` or an `Other { status: 409 }`
/// (e.g. `CommitVersionConflictException`); `http_status()` covers both.
fn is_conflict(err: &unitycatalog_client::Error) -> bool {
    matches!(err, unitycatalog_client::Error::Api(api) if api.http_status() == 409)
}

#[cfg(test)]
mod tests {
    use super::is_conflict;
    use unitycatalog_client::{Error, UcApiError};

    #[test]
    fn conflict_detects_already_exists_and_409_other() {
        // Typed AlreadyExists (409).
        assert!(is_conflict(&Error::Api(UcApiError::AlreadyExists {
            message: "commit 1 already exists".into(),
        })));
        // Untyped server error carrying status 409 (e.g. CommitVersionConflictException).
        assert!(is_conflict(&Error::Api(UcApiError::Other {
            status: 409,
            error_code: "CommitVersionConflictException".into(),
            message: "version conflict".into(),
        })));
    }

    #[test]
    fn conflict_ignores_non_409() {
        assert!(!is_conflict(&Error::Api(UcApiError::NotFound {
            message: "no table".into(),
        })));
        assert!(!is_conflict(&Error::Api(UcApiError::Other {
            status: 400,
            error_code: "INVALID_PARAMETER_VALUE".into(),
            message: "bad".into(),
        })));
        assert!(!is_conflict(&Error::Generic("boom".into())));
    }
}
