//! Unity Catalog backed catalog/schema/table resolution for DataFusion.
//!
//! These providers implement the `datafusion-catalog` async resolution traits
//! ([`AsyncCatalogProviderList`], [`AsyncCatalogProvider`],
//! [`AsyncSchemaProvider`]). At plan time DataFusion calls `resolve(refs, cfg)`,
//! which walks the query's table references, looks each one up in Unity Catalog
//! exactly once, and returns a synchronous, query-scoped catalog snapshot used
//! for the rest of planning and execution.
//!
//! Looking a table up does three things:
//! 1. fetch the [`Table`] metadata (storage location, format, id) from UC,
//! 2. vend credentials and register a per-table object store so the engine can
//!    read the table's storage location at scan time (see [`crate::storage`]),
//! 3. build a [`TableProvider`] for the table's data source format (Delta).
//!
//! Step 3 is delegated to a [`TableProviderBuilder`] supplied by the embedder,
//! because constructing a Delta provider requires log-store / engine wiring
//! that belongs to the host session rather than this generic crate.
//!
//! [`AsyncCatalogProviderList`]: datafusion::catalog::AsyncCatalogProviderList
//! [`AsyncCatalogProvider`]: datafusion::catalog::AsyncCatalogProvider
//! [`AsyncSchemaProvider`]: datafusion::catalog::AsyncSchemaProvider
//! [`Table`]: unitycatalog_common::models::tables::v1::Table

mod builder;
#[cfg(feature = "delta")]
mod delta;
mod provider;

pub use builder::{TableProviderBuilder, TableProviderError};
#[cfg(feature = "delta")]
pub use delta::DeltaTableProviderBuilder;
pub use provider::{UnityCatalogProvider, UnityCatalogProviderList, UnityCatalogSchemaProvider};
