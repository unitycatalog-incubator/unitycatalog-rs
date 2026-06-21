//! Detecting a metric view on a Unity Catalog [`Table`].
//!
//! Proto support for a dedicated `VIEW`/`METRIC_VIEW` table type and a
//! `view_definition` field is deferred (those are commented out in
//! `proto/.../tables/v1/models.proto`). Until then, a metric view is carried on
//! a regular [`Table`] via two well-known properties:
//!
//! - [`METRIC_VIEW_TYPE_PROPERTY`] = [`METRIC_VIEW_TYPE_VALUE`] marks the table
//!   as a metric view; and
//! - [`METRIC_VIEW_DEFINITION_PROPERTY`] carries the YAML definition.
//!
//! When the proto gains a first-class metric-view type, [`metric_view_of`] is
//! the single place to update.

use unitycatalog_common::models::tables::v1::Table;

use super::model::MetricView;

/// Property key marking a table as a metric view.
pub const METRIC_VIEW_TYPE_PROPERTY: &str = "unitycatalog.table_subtype";
/// Value of [`METRIC_VIEW_TYPE_PROPERTY`] indicating a metric view.
pub const METRIC_VIEW_TYPE_VALUE: &str = "METRIC_VIEW";
/// Property key carrying the metric-view YAML definition.
pub const METRIC_VIEW_DEFINITION_PROPERTY: &str = "unitycatalog.view_definition";

/// If `table` is a metric view, parse and return its definition.
///
/// Returns `Ok(None)` for a non-metric-view table, `Err` if it is marked as a
/// metric view but its YAML definition is missing or malformed.
pub fn metric_view_of(table: &Table) -> Result<Option<MetricView>, MetricViewDetectError> {
    let is_metric_view = table
        .properties
        .get(METRIC_VIEW_TYPE_PROPERTY)
        .is_some_and(|v| v.eq_ignore_ascii_case(METRIC_VIEW_TYPE_VALUE));
    if !is_metric_view {
        return Ok(None);
    }

    let yaml = table
        .properties
        .get(METRIC_VIEW_DEFINITION_PROPERTY)
        .ok_or(MetricViewDetectError::MissingDefinition)?;
    let view = MetricView::from_yaml(yaml).map_err(MetricViewDetectError::Parse)?;
    Ok(Some(view))
}

/// Failure detecting/parsing a metric view from a [`Table`].
#[derive(Debug, thiserror::Error)]
pub enum MetricViewDetectError {
    #[error(
        "table is marked as a metric view but has no '{METRIC_VIEW_DEFINITION_PROPERTY}' property"
    )]
    MissingDefinition,
    #[error("invalid metric-view YAML: {0}")]
    Parse(#[source] serde_yml::Error),
}
