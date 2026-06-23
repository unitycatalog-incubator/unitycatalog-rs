//! Deserializable model of the Unity Catalog metric-view YAML (the `1.1`
//! user-facing surface).
//!
//! A metric view is a semantic layer over a base relation: it names
//! [`Dimension`]s (group-by expressions) and [`Measure`]s (aggregate
//! expressions) as SQL strings. Unity Catalog returns this YAML as a table's
//! definition when the table is a metric view.
//!
//! Reference: <https://docs.databricks.com/aws/en/business-semantics/metric-views/yaml-reference>
//!
//! This models the field surface needed to lower a metric view into a query plan
//! and to derive its dependencies ([`super::deps`]); fields not yet exercised by
//! the lowering ([`Join`], `window`, `materialization`) are captured loosely.

use serde::Deserialize;

/// A Unity Catalog metric view, deserialized from its YAML definition.
#[derive(Debug, Clone, Deserialize)]
pub struct MetricView {
    /// Spec version, e.g. `"1.1"`. Carried through for diagnostics; the lowering
    /// does not branch on it.
    #[serde(default)]
    pub version: Option<String>,

    /// The base relation: a three-part `catalog.schema.table` name, or an inline
    /// SQL query.
    pub source: String,

    /// Optional SQL boolean predicate applied to every query against the view.
    #[serde(default)]
    pub filter: Option<String>,

    /// Star/snowflake joins.
    #[serde(default)]
    pub joins: Vec<Join>,

    /// Group-by expressions. The UC docs use `fields` as the canonical synonym
    /// for `dimensions`; accept either.
    #[serde(default, alias = "fields")]
    pub dimensions: Vec<Dimension>,

    /// Aggregate expressions.
    #[serde(default)]
    pub measures: Vec<Measure>,
}

/// A group-by expression exposed by the metric view.
#[derive(Debug, Clone, Deserialize)]
pub struct Dimension {
    /// Output column name for the dimension.
    pub name: String,
    /// SQL expression evaluated against the source relation.
    pub expr: String,
}

/// An aggregate expression exposed by the metric view.
#[derive(Debug, Clone, Deserialize)]
pub struct Measure {
    /// Output column name for the measure.
    pub name: String,
    /// SQL aggregate expression, e.g. `SUM(o_totalprice)`.
    pub expr: String,
}

/// A join in a star/snowflake metric view.
#[derive(Debug, Clone, Deserialize)]
pub struct Join {
    /// Alias for the joined relation.
    pub name: String,
    /// Joined relation: three-part name or inline SQL.
    pub source: String,
    /// SQL join condition (`on`). Mutually exclusive with `using` in practice.
    #[serde(default)]
    pub on: Option<String>,
    /// Columns to join on by name, as an alternative to `on`.
    #[serde(default)]
    pub using: Vec<String>,
    /// `many_to_one` (default) or `one_to_many`.
    #[serde(default)]
    pub cardinality: Option<String>,
}

impl MetricView {
    /// Parse a metric view from its YAML definition.
    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yml::Error> {
        serde_yml::from_str(yaml)
    }
}
