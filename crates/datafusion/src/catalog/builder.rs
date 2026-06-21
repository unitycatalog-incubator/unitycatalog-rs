use std::sync::Arc;

use datafusion::catalog::TableProvider;
use unitycatalog_common::models::tables::v1::Table;
use url::Url;

/// Error returned while turning a Unity Catalog table into a [`TableProvider`].
pub type TableProviderError = datafusion::error::DataFusionError;

/// Builds a DataFusion [`TableProvider`] for a Unity Catalog table.
///
/// The resolver ([`super::UnityCatalogSchemaProvider`]) handles UC metadata
/// lookup, credential vending, and per-table object store registration. It then
/// delegates the actual provider construction to an implementation of this
/// trait, because building a Delta provider requires log-store / engine wiring
/// owned by the host session rather than this generic crate.
///
/// Implementations receive the fully resolved storage `location` (the table's
/// `storage_location`) plus the full UC [`Table`] metadata. By the time this is
/// called, the object store serving `location` has already been registered on
/// the session's runtime, so reads at scan time succeed with vended credentials.
#[async_trait::async_trait]
pub trait TableProviderBuilder: Send + Sync + std::fmt::Debug {
    /// Build a provider for a Delta table rooted at `location`.
    async fn build_delta(
        &self,
        location: &Url,
        table: &Table,
    ) -> Result<Arc<dyn TableProvider>, TableProviderError>;

    /// Build a provider for a metric view.
    ///
    /// `view` is the parsed metric-view definition and `source` is the
    /// already-resolved provider for the view's source relation (the resolver
    /// resolves it through the same Unity Catalog path, so its credentials and
    /// object store are registered). Implementations turn `source` into a
    /// logical plan and lower the view into a
    /// [`MetricViewTableProvider`](crate::metric_view::MetricViewTableProvider).
    ///
    /// The default errors: only embedders that build a session with metric-view
    /// support (see [`crate::metric_view::session`]) can resolve metric views.
    #[cfg(feature = "metric-view")]
    async fn build_metric_view(
        &self,
        view: &crate::metric_view::MetricView,
        source: Arc<dyn TableProvider>,
        source_name: &str,
    ) -> Result<Arc<dyn TableProvider>, TableProviderError> {
        let _ = (view, source, source_name);
        Err(TableProviderError::NotImplemented(
            "this TableProviderBuilder does not support metric views".to_string(),
        ))
    }
}
