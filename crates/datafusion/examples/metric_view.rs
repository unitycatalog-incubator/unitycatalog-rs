//! Query a Unity Catalog metric view through DataFusion with Spark-style
//! `MEASURE()` late binding.
//!
//! A *metric view* is a semantic layer: a base relation annotated with
//! `dimensions` (group-by expressions) and `measures` (aggregate expressions),
//! defined as SQL strings in a YAML body. Unity Catalog returns this YAML as a
//! table's definition when the table is a metric view. This example registers
//! such a view over an in-memory `orders` table (so it runs with no external
//! services) and queries it:
//!
//! ```text
//! cargo run -p datafusion-unitycatalog --features metric-view --example metric_view
//! ```
//!
//! The query references measures through `MEASURE(...)`. The
//! `ResolveMetricView` analyzer rule rewrites the view into a concrete
//! aggregation over the source, materializing only the dimensions grouped and
//! measures referenced — mirroring Apache Spark (SPIP SPARK-54119).

use std::sync::Arc;

use datafusion::arrow::array::{Date32Array, Float64Array, StringArray};
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::datasource::MemTable;
use datafusion_unitycatalog::metric_view::{
    MetricView, MetricViewTableProvider, metric_view_context,
};

/// The metric-view YAML Unity Catalog would return for this table (UC `1.1`).
const METRIC_VIEW_YAML: &str = r#"
version: "1.1"
source: main.sales.orders
filter: o_orderdate >= '2022-01-01'
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // A session with the MEASURE() UDF and the ResolveMetricView analyzer rule
    // registered (the rule is ordered before type coercion).
    let ctx = metric_view_context();

    // 1. Register an in-memory `orders` table standing in for the view's source.
    let schema = Arc::new(Schema::new(vec![
        Field::new("o_orderdate", DataType::Date32, false),
        Field::new("o_orderstatus", DataType::Utf8, false),
        Field::new("o_totalprice", DataType::Float64, false),
    ]));
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            // 2022-01-23, 2022-01-23, 2022-01-24, 2022-01-24 (days since epoch).
            Arc::new(Date32Array::from(vec![19_015, 19_015, 19_016, 19_016])),
            Arc::new(StringArray::from(vec!["O", "F", "O", "O"])),
            Arc::new(Float64Array::from(vec![100.0, 50.0, 200.0, 75.0])),
        ],
    )?;
    ctx.register_table(
        "orders",
        Arc::new(MemTable::try_new(schema, vec![vec![batch]])?),
    )?;

    // 2. Build a metric-view provider and register it as `orders_metrics`.
    let view = MetricView::from_yaml(METRIC_VIEW_YAML)?;
    let source = ctx.table("orders").await?.into_unoptimized_plan();
    let provider = MetricViewTableProvider::try_new(&ctx.state(), &view, source)?;
    ctx.register_table("orders_metrics", Arc::new(provider))?;

    // 3. Query the view through MEASURE(). Only `total_revenue` is referenced,
    //    so only it is materialized (late binding) — `order_count` and
    //    `open_revenue` never enter the plan.
    let query = "SELECT order_date, MEASURE(total_revenue) AS revenue \
                 FROM orders_metrics GROUP BY order_date ORDER BY order_date";

    let resolved = ctx.sql(query).await?.into_optimized_plan()?;
    println!(
        "query:\n  {query}\n\nresolved plan (note: only sum() materialized):\n{}\n",
        resolved.display_indent()
    );

    let batches = ctx.sql(query).await?.collect().await?;
    println!("result:");
    datafusion::arrow::util::pretty::print_batches(&batches)?;

    Ok(())
}
