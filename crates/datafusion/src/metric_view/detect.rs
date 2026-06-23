//! Metric-view detection now lives in `unitycatalog-common` as the single source
//! of truth. Re-exported here so the DataFusion catalog resolution keeps
//! referring to `crate::metric_view::detect::*`.

pub use unitycatalog_common::metric_view::detect::{MetricViewDetectError, metric_view_of};
