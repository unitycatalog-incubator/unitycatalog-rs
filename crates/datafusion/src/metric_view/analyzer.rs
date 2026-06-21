//! The [`ResolveMetricView`] analyzer rule — Spark-style `MEASURE()` late
//! binding for DataFusion.
//!
//! After SQL planning, a query against a metric view looks like one of:
//!
//! ```text
//! # Form A: explicit aggregation over the view
//! Aggregate group=[mv.order_date], aggr=[measure(mv.total_revenue)]
//! └─ SubqueryAlias(mv)
//!    └─ MetricViewPlaceholder
//!       └─ <source>
//!
//! # Form B: a bare scan (SELECT * / SELECT order_date FROM mv)
//! ... └─ SubqueryAlias(mv) └─ MetricViewPlaceholder └─ <source>
//! ```
//!
//! This rule rewrites the placeholder into a concrete `Aggregate` over the
//! source, **materializing only the dimensions grouped and the measures
//! referenced via `MEASURE(...)`** — the late-binding property. For Form A, the
//! enclosing aggregate drives selection: its group-by columns name the
//! dimensions, its `measure(col)` calls name the measures. For Form B, with no
//! enclosing aggregate, the full set of dimensions and measures is materialized.
//!
//! Mirrors Spark's `ResolveMetricView` (SPIP SPARK-54119).

use std::sync::Arc;

use datafusion::common::config::ConfigOptions;
use datafusion::common::tree_node::{Transformed, TreeNode};
use datafusion::common::{Column, Result, plan_err};
use datafusion::logical_expr::expr::AggregateFunction;
use datafusion::logical_expr::{Aggregate, Expr, LogicalPlan, LogicalPlanBuilder, SubqueryAlias};
use datafusion::optimizer::AnalyzerRule;

use super::measure::MEASURE_UDF_NAME;
use super::placeholder::{MetricViewPlaceholder, NamedExpr};

/// Analyzer rule that resolves metric-view placeholders and `MEASURE()` markers.
#[derive(Debug, Default)]
pub struct ResolveMetricView {}

impl ResolveMetricView {
    pub fn new() -> Self {
        Self::default()
    }
}

impl AnalyzerRule for ResolveMetricView {
    fn name(&self) -> &str {
        "resolve_metric_view"
    }

    fn analyze(&self, plan: LogicalPlan, _config: &ConfigOptions) -> Result<LogicalPlan> {
        // Top-down: handle an enclosing `Aggregate` over a placeholder (Form A)
        // before its child placeholder is reached, so we can read the query's
        // selected dimensions/measures off the aggregate. Any placeholder not
        // consumed that way (Form B) is expanded to its full aggregate.
        plan.transform_down(resolve_node).map(|t| t.data)
    }
}

/// If `plan` is (or aliases) a [`MetricViewPlaceholder`], return it and the
/// alias the inlining attached (if any).
fn as_placeholder(plan: &LogicalPlan) -> Option<(&MetricViewPlaceholder, Option<&str>)> {
    match plan {
        LogicalPlan::Extension(ext) => ext
            .node
            .as_any()
            .downcast_ref::<MetricViewPlaceholder>()
            .map(|p| (p, None)),
        LogicalPlan::SubqueryAlias(SubqueryAlias { input, alias, .. }) => {
            if let LogicalPlan::Extension(ext) = input.as_ref() {
                ext.node
                    .as_any()
                    .downcast_ref::<MetricViewPlaceholder>()
                    .map(|p| (p, Some(alias.table())))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn resolve_node(node: LogicalPlan) -> Result<Transformed<LogicalPlan>> {
    // Form A: an Aggregate whose input is (an alias of) a placeholder.
    if let LogicalPlan::Aggregate(agg) = &node
        && let Some((placeholder, alias)) = as_placeholder(&agg.input)
    {
        let rewritten = resolve_aggregate_over_view(agg, placeholder)?;
        // Re-wrap under the metric view's alias so the enclosing projection's
        // qualified column references (e.g. `mv.order_date`) still resolve.
        let rewritten = reapply_alias(rewritten, alias)?;
        return Ok(Transformed::yes(rewritten));
    }

    // Form B: a bare placeholder (no enclosing aggregate consumed it). Expand to
    // the full aggregate over all dimensions + all measures.
    if let Some((placeholder, alias)) = as_placeholder(&node) {
        let full = expand_full_aggregate(placeholder)?;
        // Preserve the alias so parent column references (e.g. `mv.order_date`)
        // still resolve.
        return Ok(Transformed::yes(reapply_alias(full, alias)?));
    }

    Ok(Transformed::no(node))
}

/// Wrap `plan` in a [`SubqueryAlias`] for `alias` so a parent's qualified
/// references (`alias.col`) resolve against the rewritten plan.
fn reapply_alias(plan: LogicalPlan, alias: Option<&str>) -> Result<LogicalPlan> {
    match alias {
        Some(name) => Ok(LogicalPlan::SubqueryAlias(SubqueryAlias::try_new(
            Arc::new(plan),
            name.to_string(),
        )?)),
        None => Ok(plan),
    }
}

/// Rewrite `agg` (an aggregate over the metric view) into an aggregate over the
/// view's source, materializing only the referenced dimensions/measures.
///
/// Each rewritten expression is aliased to the *original* expression's output
/// name, so the rewritten aggregate's schema matches what the enclosing
/// projection already references — the substitution is invisible above the
/// metric view.
fn resolve_aggregate_over_view(
    agg: &Aggregate,
    placeholder: &MetricViewPlaceholder,
) -> Result<LogicalPlan> {
    // Group-by expressions must each reference a declared dimension by column
    // name; substitute the dimension's source expression, preserving the
    // original output name.
    let mut group_expr = Vec::with_capacity(agg.group_expr.len());
    for expr in &agg.group_expr {
        let Some(col) = single_column(expr) else {
            return plan_err!(
                "metric-view GROUP BY must reference a dimension by name, found: {expr}"
            );
        };
        let Some(dim) = placeholder.dimension(&col.name) else {
            return plan_err!(
                "'{}' is not a dimension of the metric view (group by a declared dimension)",
                col.name
            );
        };
        group_expr.push(dim.expr.clone().alias(expr.qualified_name().1));
    }

    // Aggregate expressions: substitute each MEASURE(measure_col) with the
    // measure's real aggregate expression. Any non-MEASURE aggregate is rejected
    // — measures are the only valid aggregations over a metric view.
    let mut aggr_expr = Vec::with_capacity(agg.aggr_expr.len());
    for expr in &agg.aggr_expr {
        let resolved = resolve_measure_expr(expr, placeholder)?;
        aggr_expr.push(resolved.alias(expr.qualified_name().1));
    }

    LogicalPlanBuilder::from(placeholder.source.as_ref().clone())
        .aggregate(group_expr, aggr_expr)?
        .build()
}

/// Resolve an aggregate expression wrapping `MEASURE(measure_col)` into the
/// measure's real aggregate expression (unaliased — the caller aliases it to the
/// original output name). Aliases on the input are unwrapped.
fn resolve_measure_expr(expr: &Expr, placeholder: &MetricViewPlaceholder) -> Result<Expr> {
    match expr {
        Expr::Alias(alias) => resolve_measure_expr(&alias.expr, placeholder),
        Expr::AggregateFunction(AggregateFunction { func, params })
            if func.name() == MEASURE_UDF_NAME =>
        {
            let [arg] = params.args.as_slice() else {
                return plan_err!("MEASURE() takes exactly one argument");
            };
            let Some(col) = single_column(arg) else {
                return plan_err!(
                    "MEASURE() argument must be a metric-view measure name, found: {arg}"
                );
            };
            let Some(measure) = placeholder.measure(&col.name) else {
                return plan_err!(
                    "'{}' is not a measure of the metric view (wrap a declared measure in MEASURE())",
                    col.name
                );
            };
            Ok(measure.expr.clone())
        }
        other => plan_err!(
            "only MEASURE(<measure>) aggregations are valid over a metric view, found: {other}"
        ),
    }
}

/// Expand a placeholder into the full aggregate over every dimension and
/// measure (Form B: a bare scan with no enclosing aggregate).
fn expand_full_aggregate(placeholder: &MetricViewPlaceholder) -> Result<LogicalPlan> {
    let group_expr: Vec<Expr> = placeholder
        .dimensions
        .iter()
        .map(named_to_aliased)
        .collect();
    let aggr_expr: Vec<Expr> = placeholder.measures.iter().map(named_to_aliased).collect();

    LogicalPlanBuilder::from(placeholder.source.as_ref().clone())
        .aggregate(group_expr, aggr_expr)?
        .build()
}

fn named_to_aliased(ne: &NamedExpr) -> Expr {
    ne.expr.clone().alias(&ne.name)
}

/// Extract the single [`Column`] a grouping/argument expression refers to, if it
/// is a plain (optionally aliased) column reference.
fn single_column(expr: &Expr) -> Option<Column> {
    match expr {
        Expr::Column(c) => Some(c.clone()),
        Expr::Alias(a) => single_column(&a.expr),
        _ => None,
    }
}
