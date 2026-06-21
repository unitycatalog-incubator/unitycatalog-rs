pub mod catalog;
#[cfg(feature = "delta")]
pub mod managed;
// Unity Catalog DDL statements + planner. The managed `CREATE TABLE` path calls
// into `managed`, so the module rides the same `delta` feature.
#[cfg(feature = "delta")]
pub mod sql;
#[cfg(feature = "metric-view")]
pub mod metric_view;
pub mod storage;

pub use self::storage::RoutingObjectStore;
