//! The [`MetricViewPlaceholder`] logical plan node.
//!
//! A metric view does **not** pre-aggregate. Instead, its [`TableProvider`] (see
//! [`super::provider`]) returns this placeholder as its logical plan, so that a
//! query against the view inlines into:
//!
//! ```text
//! <enclosing Projection / Aggregate referencing dims + MEASURE(measures)>
//! └─ SubqueryAlias(mv)
//!    └─ MetricViewPlaceholder            <- this node
//!       └─ <source plan>
//! ```
//!
//! The placeholder's output schema is the metric view's full relational surface
//! — every dimension and every measure — so name resolution and `MEASURE(col)`
//! type-checking succeed during analysis. It carries the *parsed* dimension and
//! measure expressions (resolved against the source schema) but does **not**
//! aggregate. [`super::analyzer::ResolveMetricView`] rewrites this node, plus the
//! enclosing aggregate, into the concrete `Aggregate` over the source —
//! materializing only the dimensions/measures the query actually references
//! (Spark-style late binding).
//!
//! [`TableProvider`]: datafusion::catalog::TableProvider

use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use datafusion::common::{DFSchema, DFSchemaRef, Result};
use datafusion::logical_expr::{Expr, ExprSchemable, LogicalPlan, UserDefinedLogicalNodeCore};

/// Name used in `EXPLAIN` output and node identification.
pub const METRIC_VIEW_NODE_NAME: &str = "MetricViewPlaceholder";

/// A named, parsed metric-view expression (a dimension or a measure).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NamedExpr {
    /// The declared output name (the dimension/measure `name`).
    pub name: String,
    /// The expression parsed against the source schema. For a dimension this is
    /// a scalar expression; for a measure it is an aggregate expression.
    pub expr: Expr,
}

/// Unresolved metric-view node: source + declared dimensions/measures, with the
/// view's full relational schema as output. See module docs.
#[derive(Debug, Clone)]
pub struct MetricViewPlaceholder {
    /// The resolved source relation the view aggregates over.
    pub source: Arc<LogicalPlan>,
    /// Group-by expressions, in declaration order.
    pub dimensions: Vec<NamedExpr>,
    /// Aggregate expressions, in declaration order.
    pub measures: Vec<NamedExpr>,
    /// Output schema: one field per dimension then per measure.
    schema: DFSchemaRef,
}

impl MetricViewPlaceholder {
    /// Build the placeholder, deriving the output schema from the declared
    /// dimension/measure expressions evaluated against the `source` schema.
    pub fn try_new(
        source: Arc<LogicalPlan>,
        dimensions: Vec<NamedExpr>,
        measures: Vec<NamedExpr>,
    ) -> Result<Self> {
        let source_schema = source.schema();
        let mut fields = Vec::with_capacity(dimensions.len() + measures.len());
        for ne in dimensions.iter().chain(measures.iter()) {
            // Derive the field from the expression (type + nullability) and
            // rename it to the declared dimension/measure name.
            let (_, field) = ne.expr.to_field(source_schema)?;
            let field = field.as_ref().clone().with_name(&ne.name);
            fields.push((None, Arc::new(field)));
        }
        let schema = Arc::new(DFSchema::new_with_metadata(fields, Default::default())?);
        Ok(Self {
            source,
            dimensions,
            measures,
            schema,
        })
    }

    /// Look up a measure by name (used by the analyzer to resolve `MEASURE`).
    pub fn measure(&self, name: &str) -> Option<&NamedExpr> {
        self.measures.iter().find(|m| m.name == name)
    }

    /// Look up a dimension by name.
    pub fn dimension(&self, name: &str) -> Option<&NamedExpr> {
        self.dimensions.iter().find(|d| d.name == name)
    }
}

impl UserDefinedLogicalNodeCore for MetricViewPlaceholder {
    fn name(&self) -> &str {
        METRIC_VIEW_NODE_NAME
    }

    fn inputs(&self) -> Vec<&LogicalPlan> {
        vec![self.source.as_ref()]
    }

    fn schema(&self) -> &DFSchemaRef {
        &self.schema
    }

    /// The dimension and measure expressions are carried as this node's
    /// expressions so the framework can rewrite column references inside them
    /// when the source schema changes during optimization.
    fn expressions(&self) -> Vec<Expr> {
        self.dimensions
            .iter()
            .chain(self.measures.iter())
            .map(|ne| ne.expr.clone())
            .collect()
    }

    /// Never push predicates beneath the placeholder: it represents an
    /// aggregation boundary that the analyzer has not yet materialized.
    fn prevent_predicate_push_down_columns(&self) -> HashSet<String> {
        self.schema
            .fields()
            .iter()
            .map(|f| f.name().clone())
            .collect()
    }

    fn fmt_for_explain(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let dims: Vec<&str> = self.dimensions.iter().map(|d| d.name.as_str()).collect();
        let measures: Vec<&str> = self.measures.iter().map(|m| m.name.as_str()).collect();
        write!(
            f,
            "{METRIC_VIEW_NODE_NAME}: dimensions=[{}], measures=[{}]",
            dims.join(", "),
            measures.join(", ")
        )
    }

    fn with_exprs_and_inputs(
        &self,
        exprs: Vec<Expr>,
        mut inputs: Vec<LogicalPlan>,
    ) -> Result<Self> {
        let source = Arc::new(inputs.pop().ok_or_else(|| {
            datafusion::common::DataFusionError::Internal(
                "MetricViewPlaceholder requires exactly one input".into(),
            )
        })?);

        // `exprs` are dimensions followed by measures, in the same order as
        // `expressions()` returned them; re-pair them with their names.
        let n_dims = self.dimensions.len();
        let dimensions = self
            .dimensions
            .iter()
            .zip(exprs.iter().take(n_dims))
            .map(|(ne, expr)| NamedExpr {
                name: ne.name.clone(),
                expr: expr.clone(),
            })
            .collect();
        let measures = self
            .measures
            .iter()
            .zip(exprs.iter().skip(n_dims))
            .map(|(ne, expr)| NamedExpr {
                name: ne.name.clone(),
                expr: expr.clone(),
            })
            .collect();

        Self::try_new(source, dimensions, measures)
    }
}

// Manual Eq/Ord/Hash: `UserDefinedLogicalNodeCore` requires them, and the
// derived `DFSchemaRef` does not implement `PartialOrd`/`Hash` usefully. Two
// placeholders are equal when their source, dimensions, and measures match;
// the schema is derived from those, so it need not participate.
impl PartialEq for MetricViewPlaceholder {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source
            && self.dimensions == other.dimensions
            && self.measures == other.measures
    }
}

impl Eq for MetricViewPlaceholder {}

impl PartialOrd for MetricViewPlaceholder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Order by dimensions then measures; sufficient for the framework's
        // deterministic plan comparisons.
        match self.dimensions.partial_cmp(&other.dimensions) {
            Some(Ordering::Equal) => self.measures.partial_cmp(&other.measures),
            ord => ord,
        }
    }
}

impl Hash for MetricViewPlaceholder {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.source.hash(state);
        self.dimensions.hash(state);
        self.measures.hash(state);
    }
}

// `NamedExpr` needs `PartialOrd` for the node's `PartialOrd`.
impl PartialOrd for NamedExpr {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.name.partial_cmp(&other.name)
    }
}
