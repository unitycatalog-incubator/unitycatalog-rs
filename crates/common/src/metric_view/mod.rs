//! Unity Catalog metric-view definition: the single source of truth for parsing
//! and understanding a metric view.
//!
//! A *metric view* is a semantic layer over a base relation: it names
//! `dimensions` (group-by expressions) and `measures` (aggregate expressions) as
//! SQL strings in a YAML body ([`model`]). Unity Catalog returns this YAML as a
//! table's `view_definition` when the table is a [`TableType::MetricView`].
//!
//! This module owns three responsibilities, shared by every crate that needs to
//! understand a metric view:
//!
//! 1. [`model`] — the deserializable YAML model ([`MetricView`] and friends).
//! 2. [`detect`] — recognizing a metric view on a [`Table`] and parsing it.
//! 3. [`deps`] — deriving the view's [`DependencyList`] (the tables/functions it
//!    reads) from the definition, using [`sqlparser`] for non-trivial sources.
//!
//! The catalog server uses (2)+(3) to derive and store `view_dependencies` on
//! create; the DataFusion integration re-exports this module and reuses it to
//! lower the view into a query plan. There is exactly one parser.
//!
//! [`Table`]: crate::models::tables::v1::Table
//! [`TableType::MetricView`]: crate::models::tables::v1::TableType::MetricView
//! [`DependencyList`]: crate::models::tables::v1::DependencyList

pub mod deps;
pub mod detect;
pub mod model;

pub use deps::{DependencyError, dependencies};
pub use detect::{MetricViewDetectError, metric_view_of};
pub use model::{Dimension, Join, Measure, MetricView};
