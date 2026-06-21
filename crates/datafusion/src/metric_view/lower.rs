//! Build a [`MetricViewPlaceholder`] from a [`MetricView`] definition.
//!
//! Parsing the metric-view YAML's dimension/measure/filter SQL strings into
//! DataFusion [`Expr`]s happens here, against the source relation's schema. The
//! result is the unresolved [`MetricViewPlaceholder`] node; the actual
//! aggregation is deferred to [`super::analyzer::ResolveMetricView`] at query
//! time (Spark-style `MEASURE()` late binding). See the module docs in
//! [`super`].

use std::sync::Arc;

use datafusion::common::{DFSchema, Result, plan_err};
use datafusion::execution::session_state::SessionState;
use datafusion::logical_expr::{LogicalPlan, LogicalPlanBuilder};

use super::model::MetricView;
use super::placeholder::{MetricViewPlaceholder, NamedExpr};

/// Lower `view` over its resolved `source` plan into a [`MetricViewPlaceholder`].
///
/// `state` supplies the SQL dialect and expression planner used to parse each
/// dimension/measure/filter SQL string into an [`Expr`](datafusion::logical_expr::Expr)
/// resolved against the source schema. The view's `filter`, if any, is applied
/// beneath the placeholder so every query sees it.
pub fn build_placeholder(
    state: &SessionState,
    view: &MetricView,
    source: LogicalPlan,
) -> Result<MetricViewPlaceholder> {
    if view.dimensions.is_empty() && view.measures.is_empty() {
        return plan_err!(
            "metric view '{}' defines neither dimensions nor measures",
            view.source
        );
    }
    if !view.joins.is_empty() {
        return plan_err!(
            "metric-view joins are not yet supported (source '{}' declares {} join(s))",
            view.source,
            view.joins.len()
        );
    }

    // The view filter applies beneath the aggregation, exactly as Spark places
    // the view predicate below the grouping. Fold it into the source plan so it
    // travels with the placeholder's input.
    let source = match &view.filter {
        Some(filter) => {
            let predicate = state.create_logical_expr(filter, source.schema())?;
            LogicalPlanBuilder::from(source)
                .filter(predicate)?
                .build()?
        }
        None => source,
    };
    let source_schema: &DFSchema = source.schema();

    let dimensions = view
        .dimensions
        .iter()
        .map(|d| {
            state
                .create_logical_expr(&d.expr, source_schema)
                .map(|expr| NamedExpr {
                    name: d.name.clone(),
                    expr,
                })
        })
        .collect::<Result<Vec<_>>>()?;

    let measures = view
        .measures
        .iter()
        .map(|m| {
            state
                .create_logical_expr(&m.expr, source_schema)
                .map(|expr| NamedExpr {
                    name: m.name.clone(),
                    expr,
                })
        })
        .collect::<Result<Vec<_>>>()?;

    MetricViewPlaceholder::try_new(Arc::new(source), dimensions, measures)
}
