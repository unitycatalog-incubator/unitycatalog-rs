//! A Delta-backed [`TableProviderBuilder`].
//!
//! Enabled by the `delta` feature. This builds a DataFusion [`TableProvider`]
//! for a Unity Catalog Delta table by reading its log through whatever object
//! store the resolver has already registered on the session runtime for the
//! table's storage location.

use std::sync::Arc;

use datafusion::catalog::TableProvider;
use datafusion::common::DataFusionError;
use datafusion::execution::context::SessionContext;
use deltalake_core::DeltaTableConfig;
use deltalake_core::delta_datafusion::DeltaScanNext;
use deltalake_core::delta_datafusion::engine::DataFusionEngine;
use deltalake_core::kernel::Snapshot;
use deltalake_core::logstore::{StorageConfig, logstore_with};
use unitycatalog_common::models::tables::v1::Table;
use url::Url;

use super::builder::{TableProviderBuilder, TableProviderError};

/// Builds Delta [`TableProvider`]s for Unity Catalog tables.
///
/// Holds a [`SessionContext`] so it can read through the runtime's object store
/// registry — by the time [`build_delta`](TableProviderBuilder::build_delta) is
/// called, the resolver has already registered a credential-vended store for the
/// table's storage location, so the log scan succeeds against that store.
#[derive(Clone)]
pub struct DeltaTableProviderBuilder {
    ctx: SessionContext,
}

impl DeltaTableProviderBuilder {
    /// Create a builder that resolves Delta tables through `ctx`'s runtime.
    pub fn new(ctx: SessionContext) -> Self {
        Self { ctx }
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
        _table: &Table,
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
        let snapshot = Snapshot::try_new_with_engine(
            engine,
            location.clone(),
            DeltaTableConfig::default(),
            None,
        )
        .await
        .map_err(|e| DataFusionError::External(Box::new(e)))?;

        let provider = DeltaScanNext::builder()
            .with_snapshot(Arc::new(snapshot))
            .with_log_store(log_store)
            .await?;
        Ok(provider)
    }
}
