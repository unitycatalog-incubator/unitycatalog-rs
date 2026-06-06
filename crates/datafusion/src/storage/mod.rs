//! Object store wiring for Unity Catalog backed tables.
//!
//! Unity Catalog vends temporary credentials that are scoped to a *sub-path*
//! of a bucket (the table's storage location), but DataFusion's
//! [`ObjectStoreRegistry`] resolves stores by `scheme://host` only — the path
//! is discarded. Two tables in the same bucket therefore collide on a single
//! registry key. [`RoutingObjectStore`] sits behind that coarse key and
//! dispatches each operation to the correct per-table store based on the full
//! request path.
//!
//! [`ObjectStoreRegistry`]: datafusion::execution::object_store::ObjectStoreRegistry

mod routing;

pub use routing::RoutingObjectStore;
