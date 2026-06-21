pub mod catalog;
#[cfg(feature = "delta")]
pub mod managed;
#[cfg(feature = "metric-view")]
pub mod metric_view;
pub mod storage;

pub use self::storage::RoutingObjectStore;
