//! A Delta-backed [`TableProviderBuilder`].
//!
//! Enabled by the `delta` feature. This builds a DataFusion [`TableProvider`]
//! for a Unity Catalog Delta table by reading its log through whatever object
//! store the resolver has already registered on the session runtime for the
//! table's storage location.
//!
//! For **catalog-managed** (coordinated-commit) tables the filesystem `_delta_log/`
//! is not authoritative: the catalog tracks the latest ratified version and the
//! newest commits may be unbackfilled. So the builder calls the `/delta/v1`
//! `loadTable` endpoint to fetch the commit tail + latest version and builds a
//! catalog-managed kernel snapshot from them (see [`super::kernel`]). External
//! Delta tables keep the plain filesystem snapshot path.

use std::sync::Arc;

use datafusion::catalog::TableProvider;
use datafusion::common::DataFusionError;
use datafusion::execution::context::SessionContext;
use deltalake_core::DeltaTableConfig;
use deltalake_core::delta_datafusion::DeltaScanNext;
use deltalake_core::delta_datafusion::engine::DataFusionEngine;
use deltalake_core::kernel::Snapshot;
use deltalake_core::logstore::{StorageConfig, logstore_with};
use unitycatalog_client::UnityCatalogClient;
use unitycatalog_common::models::delta::v1::DeltaTableType;
use unitycatalog_common::models::tables::v1::Table;
use url::Url;

use super::builder::{TableProviderBuilder, TableProviderError};
use super::kernel::build_catalog_managed_snapshot;

/// Builds Delta [`TableProvider`]s for Unity Catalog tables.
///
/// Holds a [`SessionContext`] so it can read through the runtime's object store
/// registry — by the time [`build_delta`](TableProviderBuilder::build_delta) is
/// called, the resolver has already registered a credential-vended store for the
/// table's storage location, so the log scan succeeds against that store. Also
/// holds a [`UnityCatalogClient`] to call the `/delta/v1` `loadTable` endpoint for
/// catalog-managed tables.
#[derive(Clone)]
pub struct DeltaTableProviderBuilder {
    ctx: SessionContext,
    client: UnityCatalogClient,
}

impl DeltaTableProviderBuilder {
    /// Create a builder that resolves Delta tables through `ctx`'s runtime and
    /// loads catalog-managed metadata through `client`.
    pub fn new(ctx: SessionContext, client: UnityCatalogClient) -> Self {
        Self { ctx, client }
    }
}

impl std::fmt::Debug for DeltaTableProviderBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeltaTableProviderBuilder")
            .finish_non_exhaustive()
    }
}

#[async_trait::async_trait]
impl TableProviderBuilder for DeltaTableProviderBuilder {
    async fn build_delta(
        &self,
        location: &Url,
        table: &Table,
    ) -> Result<Arc<dyn TableProvider>, TableProviderError> {
        let task_ctx = self.ctx.task_ctx();
        let root_store = self
            .ctx
            .runtime_env()
            .object_store_registry
            .get_store(location)
            .map_err(|e| DataFusionError::External(Box::new(e)))?;
        let log_store = logstore_with(root_store, location, StorageConfig::default())
            .map_err(|e| DataFusionError::External(Box::new(e)))?;

        let engine = DataFusionEngine::new_from_context(task_ctx);

        // Ask the catalog whether this is a managed (coordinated-commit) table and,
        // if so, for its ratified commit tail. The `/delta/v1` loadTable response
        // carries the table type, the unbackfilled commits, and the latest ratified
        // version a reader needs to materialize the catalog's snapshot.
        let loaded = self
            .client
            .delta_v1()
            .load_table(&table.catalog_name, &table.schema_name, &table.name)
            .await
            .map_err(|e| DataFusionError::External(Box::new(e)))?;

        let snapshot = match loaded.metadata.table_type {
            DeltaTableType::Managed => {
                // The catalog is the source of truth: build from the ratified commit
                // tail + latest version rather than scanning `_delta_log/`.
                let commits = loaded.commits.as_deref().unwrap_or(&[]);
                let latest = loaded
                    .latest_table_version
                    .unwrap_or(loaded.metadata.last_commit_version.unwrap_or(0));
                build_catalog_managed_snapshot(engine.as_ref(), location, commits, latest, None)?
            }
            DeltaTableType::External => {
                // External tables track version on the filesystem; the legacy
                // (non-catalog-managed) path must not set a catalog version.
                Snapshot::try_new_with_engine(
                    engine,
                    location.clone(),
                    DeltaTableConfig::default(),
                    None,
                )
                .await
                .map_err(|e| DataFusionError::External(Box::new(e)))?
            }
        };

        let provider = DeltaScanNext::builder()
            .with_snapshot(Arc::new(snapshot))
            .with_log_store(log_store)
            .await?;
        Ok(provider)
    }
}
