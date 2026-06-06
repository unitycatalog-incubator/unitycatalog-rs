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
}
