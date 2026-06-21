//! Registering metric-view support on a DataFusion session.
//!
//! Two things must be installed for metric views to resolve:
//!
//! 1. the [`MEASURE()`](super::measure) marker aggregate UDF, so queries that
//!    reference measures plan; and
//! 2. the [`ResolveMetricView`] analyzer rule, which must run **before**
//!    `TypeCoercion` — after the rule rewrites `MEASURE(...)` into real
//!    aggregate expressions, those expressions still need coercion.
//!
//! Since `SessionState::add_analyzer_rule` appends (running after type
//! coercion), this module rebuilds the analyzer rule list with
//! [`ResolveMetricView`] prepended.

use std::sync::Arc;

use datafusion::execution::registry::FunctionRegistry;
use datafusion::execution::session_state::{SessionState, SessionStateBuilder};
use datafusion::optimizer::AnalyzerRule;
use datafusion::optimizer::analyzer::Analyzer;
use datafusion::prelude::SessionContext;

use super::analyzer::ResolveMetricView;
use super::measure::measure_udf;

/// Return the analyzer rules for metric-view support: [`ResolveMetricView`]
/// prepended to DataFusion's default rules so it runs before type coercion.
fn metric_view_analyzer_rules() -> Vec<Arc<dyn AnalyzerRule + Send + Sync>> {
    let mut rules: Vec<Arc<dyn AnalyzerRule + Send + Sync>> =
        vec![Arc::new(ResolveMetricView::new())];
    rules.extend(Analyzer::new().rules);
    rules
}

/// Build a [`SessionState`] with metric-view support registered: the `MEASURE()`
/// UDF and the [`ResolveMetricView`] analyzer rule (ordered before type
/// coercion).
pub fn metric_view_session_state() -> SessionState {
    let mut state = SessionStateBuilder::new()
        .with_default_features()
        .with_analyzer_rules(metric_view_analyzer_rules())
        .build();
    // Registering on the built state preserves the built-in aggregates that
    // `with_aggregate_functions` would otherwise replace.
    state
        .register_udaf(measure_udf())
        .expect("register MEASURE() UDF");
    state
}

/// Build a [`SessionContext`] with metric-view support registered.
pub fn metric_view_context() -> SessionContext {
    SessionContext::new_with_state(metric_view_session_state())
}
