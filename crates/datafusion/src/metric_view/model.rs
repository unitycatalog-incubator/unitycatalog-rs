//! The metric-view YAML model now lives in `unitycatalog-common` as the single
//! source of truth (shared with the catalog server, which derives
//! `view_dependencies` from it). Re-exported here so the DataFusion lowering
//! keeps referring to `crate::metric_view::model::*`.

pub use unitycatalog_common::metric_view::model::{Dimension, Join, Measure, MetricView};
