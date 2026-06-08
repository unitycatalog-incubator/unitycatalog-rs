//! Unity Catalog catalog-managed Delta table support: create + commit through the
//! kernel's [`Committer`](delta_kernel::committer::Committer) framework, backed by the
//! unitycatalog-rs [`DeltaV1Client`](unitycatalog_client::DeltaV1Client).
//!
//! - [`UnityCatalogCommitter`] — the catalog-managed committer (stage → ratify → publish).
//! - `create_managed_table` (Phase 2) — the staging → `0.json` → `createTable` connector.

mod committer;
mod create;

pub use committer::UnityCatalogCommitter;
pub use create::{
    create_managed_table, get_final_required_properties_for_uc, get_required_properties_for_disk,
    CreateManagedTableError, ManagedTable,
};
