//! Unity Catalog metric views as DataFusion plans, with Spark-style `MEASURE()`
//! late binding.
//!
//! A *metric view* is a semantic layer over a base relation: it names
//! `dimensions` (group-by expressions) and `measures` (aggregate expressions) as
//! SQL strings in a YAML body ([`model`]). Unity Catalog returns this YAML as a
//! table's definition when the table is a metric view.
//!
//! # How a query against a metric view resolves
//!
//! Unlike a plain view, a metric view does not pre-aggregate. Only the
//! dimensions a query groups by and the measures it references (via the
//! `MEASURE(...)` marker) are materialized — *late binding*, matching Apache
//! Spark (SPIP SPARK-54119):
//!
//! ```sql
//! SELECT order_date, MEASURE(total_revenue)
//! FROM   main.sales.orders_metrics
//! GROUP BY order_date
//! ```
//!
//! The pipeline:
//!
//! 1. [`MetricViewTableProvider`] (built by [`build_placeholder`] +
//!    [`MetricViewTableProvider::try_new`]) exposes the view's full
//!    dimension+measure schema and returns a [`MetricViewPlaceholder`] from
//!    `get_logical_plan`. The SQL planner inlines it in place of the scan.
//! 2. [`measure`] registers `MEASURE()` as a marker aggregate UDF so the query
//!    plans.
//! 3. [`ResolveMetricView`] (an analyzer rule) rewrites the placeholder + the
//!    enclosing aggregate into a concrete `Aggregate` over the source,
//!    substituting each `MEASURE(m)` with the measure's real aggregate
//!    expression and materializing only the referenced dimensions/measures.
//! 4. [`session`] wires (2) and (3) onto a [`SessionContext`] in the correct
//!    order (the rule must precede type coercion).
//!
//! Use [`metric_view_context`] to get a session with everything registered.
//!
//! [`SessionContext`]: datafusion::prelude::SessionContext

pub mod analyzer;
pub mod detect;
pub mod lower;
pub mod measure;
pub mod model;
pub mod placeholder;
pub mod provider;
pub mod session;

#[cfg(test)]
mod tests;

pub use analyzer::ResolveMetricView;
pub use detect::metric_view_of;
pub use lower::build_placeholder;
pub use measure::{MEASURE_UDF_NAME, measure_udf};
pub use model::{Dimension, Join, Measure, MetricView};
// Dependency derivation is owned by `unitycatalog-common`; re-export so the
// DataFusion path can validate the relations it resolves against the same logic.
pub use placeholder::{MetricViewPlaceholder, NamedExpr};
pub use provider::MetricViewTableProvider;
pub use session::{metric_view_context, metric_view_session_state};
pub use unitycatalog_common::metric_view::{DependencyError, dependencies};
