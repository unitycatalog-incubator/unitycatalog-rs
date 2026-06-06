//! Query a Unity Catalog Delta table through DataFusion.
//!
//! This wires the three pieces this crate provides into a DataFusion session:
//!
//! 1. Create a [`SessionContext`].
//! 2. Register a UC-backed routing object store on the session's runtime. We
//!    don't do this by hand — [`UnityCatalogProviderList`] owns the session's
//!    [`RuntimeEnv`] and registers a per-bucket [`RoutingObjectStore`] for each
//!    table it resolves, vending credentials lazily at resolution time.
//! 3. Register the resolved catalog list on the session, then plan and execute.
//!
//! The Delta provider construction is delegated to [`DeltaTableProviderBuilder`]
//! (the `delta` feature), exactly as the integration tests do.
//!
//! Run it against a live UC instance:
//!
//! ```text
//! UC_ENDPOINT=http://localhost:8081/api/2.1/unity-catalog/ \
//! UC_TABLE=unity.default.numbers \
//! cargo run -p datafusion-unitycatalog --features delta --example unity_catalog
//! ```
//!
//! Set `UC_TOKEN` for an authenticated server (omit it for a local
//! unauthenticated OSS server) and `AWS_REGION` if the storage backend needs it.

use std::sync::Arc;

use datafusion::catalog::AsyncCatalogProviderList;
use datafusion::common::TableReference;
use datafusion::prelude::SessionContext;
use datafusion_unitycatalog::catalog::{DeltaTableProviderBuilder, UnityCatalogProviderList};
use unitycatalog_object_store::UnityObjectStoreFactory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let endpoint = std::env::var("UC_ENDPOINT")
        .expect("set UC_ENDPOINT to your Unity Catalog instance, e.g. http://localhost:8081/api/2.1/unity-catalog/");
    let full_name = std::env::var("UC_TABLE").expect("set UC_TABLE=<catalog>.<schema>.<table>");

    // 1. Create the DataFusion session.
    let ctx = SessionContext::new();

    // Build the UC object store factory used for metadata lookups and
    // credential vending. Auth mirrors the integration tests: a bearer token if
    // present, otherwise allow an unauthenticated local server.
    let mut factory = UnityObjectStoreFactory::builder().with_uri(endpoint);
    match std::env::var("UC_TOKEN") {
        Ok(token) => factory = factory.with_token(token),
        Err(_) => factory = factory.with_allow_unauthenticated(true),
    }
    if let Ok(region) = std::env::var("AWS_REGION") {
        factory = factory.with_aws_region(region);
    }
    let factory = Arc::new(factory.build().await?);

    // The Delta provider builder reads each table's log through whatever store
    // the resolver registers on the runtime for that table's location.
    let builder = Arc::new(DeltaTableProviderBuilder::new(ctx.clone()));

    // 2. The provider list owns the session's runtime, so resolving a table
    //    registers the credential-vended routing store on the session.
    let providers = UnityCatalogProviderList::new(factory, ctx.runtime_env(), builder);

    // Drive async resolution exactly as the session does at plan time: look the
    // reference up once, registering its object store as a side effect.
    let reference = TableReference::parse_str(&full_name);
    let resolved = providers
        .resolve(std::slice::from_ref(&reference), &ctx.copied_config())
        .await?;

    // 3. Register the resolved (synchronous, query-scoped) catalog list on the
    //    session and run a query against the table.
    ctx.register_catalog_list(resolved);

    let df = ctx
        .sql(&format!("SELECT * FROM {full_name} LIMIT 10"))
        .await?;
    let batches = df.collect().await?;
    let rows: usize = batches.iter().map(|b| b.num_rows()).sum();
    println!("scanned {rows} rows from {full_name}");
    datafusion::arrow::util::pretty::print_batches(&batches)?;

    Ok(())
}
