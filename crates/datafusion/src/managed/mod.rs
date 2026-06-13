//! Unity Catalog catalog-managed Delta table support: create + commit through the
//! kernel's [`Committer`](delta_kernel::committer::Committer) framework, backed by the
//! unitycatalog-rs [`DeltaV1Client`](unitycatalog_client::DeltaV1Client).
//!
//! - [`UnityCatalogCommitter`] — the catalog-managed committer (v0 publish, v≥1 stage + ratify).
//! - [`create_managed_table`] — staging → `kernel::create_table` (writes `0.json`) → `createTable`.
//! - [`append_to_managed_table`] — load snapshot → kernel write transaction → commit (v≥1)
//!   with bounded conflict/throttle/ambiguity retry, then best-effort publish + backfill +
//!   metrics (ManagedTablesSpec §"Write to the table").
//!
//! Design + rationale: see open-lakehouse `docs/adr/0010-catalog-managed-table-writes.md`.
//!
//! ## Upstream / follow-ups (tracked here so they travel with the code)
//!
//! - **delta-rs `V2Checkpoint` allow-list**: this module requires a delta-rs that allow-lists
//!   the `v2Checkpoint` table feature in `ProtocolChecker` (it's part of the UC managed-table
//!   contract). The workspace pins the `roeap/delta-rs` fork rev carrying that change. Upstream
//!   it to delta-rs (a focused PR, not the "demo updates" commit it currently rides on), then
//!   bump the pin off the fork rev.
//! - **Schema/property evolution on append**: ALTER-style commits (protocol/metadata/clustering
//!   changes) are rejected, not propagated — see `UnityCatalogCommitter`'s
//!   `validate_no_alter_table_changes`. Lifting this requires bundling
//!   `set-columns`/`set-properties`/`set-protocol` actions into the same `updateTable` call as
//!   `add-commit`.
//! - **Upstream home**: keep this glue dependency-light so it can move toward
//!   `delta-rs/crates/catalog-unity`, or adopt the buoyant `delta-kernel-unity-catalog` /
//!   `unity-catalog-delta-rest-client` crates once they stabilize.

mod append;
mod committer;
mod create;

pub use append::append_to_managed_table;
pub use committer::UnityCatalogCommitter;
pub use create::{
    CreateManagedTableError, ManagedTable, create_managed_table,
    get_final_required_properties_for_uc, get_required_properties_for_disk,
};
