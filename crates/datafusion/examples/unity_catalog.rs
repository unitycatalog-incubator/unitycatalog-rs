//! Query a Unity Catalog Delta table through DataFusion.
//!
//! Unity Catalog is a *remote, asynchronous* catalog: resolving a table means a
//! network round-trip to fetch metadata and vend credentials. DataFusion's
//! high-level [`SessionContext::sql`] cannot drive async catalog resolution, so
//! we follow the same pattern as DataFusion's own "Remote Catalog" example
//! (`datafusion-examples/examples/data_io/remote_catalog.rs`): parse the query,
//! extract the table references it touches, resolve exactly those, then plan and
//! execute.
//!
//! 1. Create a [`SessionContext`] and the UC-backed [`UnityCatalogProviderList`].
//!    The provider list owns the session's [`RuntimeEnv`]; resolving a table
//!    vends credentials lazily and registers a per-bucket [`RoutingObjectStore`]
//!    on the runtime as a side effect — we never wire object stores by hand.
//! 2. Parse the SQL into a statement (without planning), then call
//!    [`SessionState::resolve_table_references`] to discover which tables the
//!    query references.
//! 3. Asynchronously [`resolve`] those references against Unity Catalog, register
//!    the resulting (synchronous, query-scoped) catalog list on the session, and
//!    plan/execute the already-parsed statement.
//!
//! The Delta provider construction is delegated to [`DeltaTableProviderBuilder`]
//! (the `delta` feature), exactly as the integration tests do.
//!
//! Run it against a live UC instance:
//!
//! ```text
//! UC_ENDPOINT=http://localhost:8081/api/2.1/unity-catalog/ \
//! UC_QUERY='SELECT * FROM unity.default.numbers LIMIT 10' \
//! cargo run -p datafusion-unitycatalog --features delta --example unity_catalog
//! ```
//!
//! Set `UC_TOKEN` for an authenticated server (omit it for a local
//! unauthenticated OSS server) and `AWS_REGION` if the storage backend needs it.
//!
//! [`resolve`]: datafusion::catalog::AsyncCatalogProviderList::resolve
//! [`SessionState::resolve_table_references`]: datafusion::execution::SessionState::resolve_table_references

use std::sync::Arc;

use datafusion::catalog::AsyncCatalogProviderList;
use datafusion::prelude::{DataFrame, SessionContext};
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
    let sql = std::env::var("UC_QUERY").expect(
        "set UC_QUERY to a full SQL statement, e.g. 'SELECT * FROM unity.default.numbers LIMIT 10'",
    );

    // 1. Create the DataFusion session.
    let ctx = SessionContext::new();

    // Build the UC object store factory used for metadata lookups and
    // credential vending. Auth mirrors the integration tests: a bearer token if
    // present, otherwise allow an unauthenticated local server.
    let mut builder = UnityObjectStoreFactory::builder().with_uri(endpoint);
    match std::env::var("UC_TOKEN") {
        Ok(token) => builder = builder.with_token(token),
        Err(_) => builder = builder.with_allow_unauthenticated(true),
    }
    if let Ok(region) = std::env::var("AWS_REGION") {
        builder = builder.with_aws_region(region);
    }
    let factory = Arc::new(builder.build().await?);

    // The Delta provider builder reads each table's log through whatever store
    // the resolver registers on the runtime for that table's location.
    let builder = Arc::new(DeltaTableProviderBuilder::new(ctx.clone()));

    // The provider list owns the session's runtime, so resolving a table
    // registers the credential-vended routing store on the session as a side
    // effect.
    let providers = UnityCatalogProviderList::new(factory, ctx.runtime_env(), builder);

    // `ctx.sql(&sql)` cannot drive async catalog resolution, so we use the same
    // lower-level `SessionState` APIs that DataFusion's "Remote Catalog" example
    // uses (it is what `ctx.sql` does internally).
    let state = ctx.state();

    // 2. Parse the SQL into a statement without planning or resolving tables.
    let dialect = state.config().options().sql_parser.dialect;
    let statement = state.sql_to_statement(&sql, &dialect)?;

    // Discover which tables the query actually references.
    let references = state.resolve_table_references(&statement)?;

    // 3. Asynchronously resolve exactly those references against Unity Catalog.
    //    This looks each reference up once, vending credentials and registering
    //    the per-table routing object store on the runtime as it goes, and yields
    //    a synchronous, query-scoped catalog list.
    let resolved = providers.resolve(&references, state.config()).await?;

    // Register the resolved catalog list, then plan and execute the parsed
    // statement against a fresh state that sees it.
    ctx.register_catalog_list(resolved);
    let plan = ctx.state().statement_to_plan(statement).await?;
    let batches = DataFrame::new(ctx.state(), plan).collect().await?;

    let rows: usize = batches.iter().map(|b| b.num_rows()).sum();
    println!("scanned {rows} rows for: {sql}");
    datafusion::arrow::util::pretty::print_batches(&batches)?;

    Ok(())
}
