//! End-to-end tests for metric-view late binding: register a metric view as a
//! table, query it through `MEASURE()`, and assert both the resolved plan shape
//! and the executed result.

use std::sync::Arc;

use datafusion::arrow::array::{Date32Array, Float64Array, StringArray};
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::assert_batches_sorted_eq;
use datafusion::datasource::MemTable;
use datafusion::prelude::SessionContext;

use super::model::MetricView;
use super::provider::MetricViewTableProvider;
use super::session::metric_view_context;

const ORDERS_METRICS_YAML: &str = r#"
version: "1.1"
source: main.sales.orders
dimensions:
  - name: order_date
    expr: o_orderdate
  - name: status
    expr: o_orderstatus
measures:
  - name: order_count
    expr: COUNT(1)
  - name: total_revenue
    expr: SUM(o_totalprice)
  - name: open_revenue
    expr: SUM(o_totalprice) FILTER (WHERE o_orderstatus = 'O')
"#;

/// Register an in-memory `orders` source and a metric view `orders_metrics` over
/// it on a metric-view-enabled context.
async fn ctx_with_metric_view(yaml: &str) -> SessionContext {
    let ctx = metric_view_context();

    let schema = Arc::new(Schema::new(vec![
        Field::new("o_orderdate", DataType::Date32, false),
        Field::new("o_orderstatus", DataType::Utf8, false),
        Field::new("o_totalprice", DataType::Float64, false),
    ]));
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            // 2022-01-08, 2022-01-08, 2022-01-09, 2022-01-09
            Arc::new(Date32Array::from(vec![19_000, 19_000, 19_001, 19_001])),
            Arc::new(StringArray::from(vec!["O", "F", "O", "O"])),
            Arc::new(Float64Array::from(vec![100.0, 50.0, 200.0, 75.0])),
        ],
    )
    .unwrap();
    let source_table = MemTable::try_new(schema, vec![vec![batch]]).unwrap();
    ctx.register_table("orders", Arc::new(source_table))
        .unwrap();

    let view = MetricView::from_yaml(yaml).unwrap();
    let source = ctx.table("orders").await.unwrap().into_unoptimized_plan();
    let provider = MetricViewTableProvider::try_new(&ctx.state(), &view, source).unwrap();
    ctx.register_table("orders_metrics", Arc::new(provider))
        .unwrap();
    ctx
}

#[tokio::test]
async fn late_binds_only_referenced_measure() {
    let ctx = ctx_with_metric_view(ORDERS_METRICS_YAML).await;

    // Query references exactly one measure and one dimension.
    const QUERY: &str =
        "SELECT order_date, MEASURE(total_revenue) AS rev FROM orders_metrics GROUP BY order_date";

    // The resolved plan must aggregate ONLY total_revenue, not the other
    // measures — this is the late-binding property. `into_optimized_plan` runs
    // the analyzer (and our ResolveMetricView rule) before the optimizer.
    let plan = ctx.sql(QUERY).await.unwrap().into_optimized_plan().unwrap();
    let rendered = format!("{}", plan.display_indent());
    assert!(
        rendered.contains("sum(orders.o_totalprice)"),
        "expected the referenced measure in the plan:\n{rendered}"
    );
    assert!(
        !rendered.contains("count(Int64(1))"),
        "order_count was not referenced and must not be materialized:\n{rendered}"
    );
    // The MEASURE() aggregate call must be resolved away into a real aggregate.
    // (The original column *label* `measure(...)` may survive as an alias string;
    // what matters is there is no MEASURE aggregate function in the plan.)
    assert!(
        !rendered.contains("aggr=[[measure("),
        "MEASURE() aggregate call should be resolved away:\n{rendered}"
    );

    let batches = ctx.sql(QUERY).await.unwrap().collect().await.unwrap();
    assert_batches_sorted_eq!(
        [
            "+------------+-------+",
            "| order_date | rev   |",
            "+------------+-------+",
            "| 2022-01-08 | 150.0 |",
            "| 2022-01-09 | 275.0 |",
            "+------------+-------+",
        ],
        &batches
    );
}

#[tokio::test]
async fn multiple_measures_and_dimensions() {
    let ctx = ctx_with_metric_view(ORDERS_METRICS_YAML).await;

    let batches = ctx
        .sql(
            "SELECT status, MEASURE(order_count) AS n, MEASURE(open_revenue) AS open \
             FROM orders_metrics GROUP BY status",
        )
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();

    assert_batches_sorted_eq!(
        [
            "+--------+---+-------+",
            "| status | n | open  |",
            "+--------+---+-------+",
            "| F      | 1 |       |",
            "| O      | 3 | 375.0 |",
            "+--------+---+-------+",
        ],
        &batches
    );
}

#[tokio::test]
async fn filter_on_resolved_measure_works() {
    let ctx = ctx_with_metric_view(ORDERS_METRICS_YAML).await;

    // A HAVING-style outer filter over the resolved aggregate.
    let batches = ctx
        .sql(
            "SELECT order_date, MEASURE(total_revenue) AS rev \
             FROM orders_metrics GROUP BY order_date HAVING MEASURE(total_revenue) > 200",
        )
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();

    assert_batches_sorted_eq!(
        [
            "+------------+-------+",
            "| order_date | rev   |",
            "+------------+-------+",
            "| 2022-01-09 | 275.0 |",
            "+------------+-------+",
        ],
        &batches
    );
}

#[tokio::test]
async fn view_filter_applies_to_every_query() {
    // The view's own filter restricts to fulfilled orders.
    let yaml = r#"
version: "1.1"
source: main.sales.orders
filter: o_orderstatus = 'F'
dimensions:
  - name: status
    expr: o_orderstatus
measures:
  - name: revenue
    expr: SUM(o_totalprice)
"#;
    let ctx = ctx_with_metric_view(yaml).await;

    let batches = ctx
        .sql("SELECT status, MEASURE(revenue) AS r FROM orders_metrics GROUP BY status")
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();

    assert_batches_sorted_eq!(
        [
            "+--------+------+",
            "| status | r    |",
            "+--------+------+",
            "| F      | 50.0 |",
            "+--------+------+",
        ],
        &batches
    );
}

#[tokio::test]
async fn bare_scan_materializes_full_aggregate() {
    // A bare scan with no enclosing aggregate (Form B) materializes every
    // dimension and measure — useful for `SELECT *`-style introspection. A
    // projection over it picks columns from the fully-grouped result.
    //
    // NOTE: Spark *rejects* referencing a measure column outside MEASURE().
    // Enforcing that strict rule is a documented follow-up; today the bare form
    // is defined as "group by all dimensions, expose all measures".
    let ctx = ctx_with_metric_view(ORDERS_METRICS_YAML).await;

    let batches = ctx
        .sql("SELECT order_date, status, total_revenue FROM orders_metrics")
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();

    assert_batches_sorted_eq!(
        [
            "+------------+--------+---------------+",
            "| order_date | status | total_revenue |",
            "+------------+--------+---------------+",
            "| 2022-01-08 | F      | 50.0          |",
            "| 2022-01-08 | O      | 100.0         |",
            "| 2022-01-09 | O      | 275.0         |",
            "+------------+--------+---------------+",
        ],
        &batches
    );
}
