//! Integration test for the Unity Catalog DataFusion resolver.
//!
//! These tests hit a live Unity Catalog server and its backing object store, so
//! they are `#[ignore]`d by default. They also use the crate's Delta
//! [`DeltaTableProviderBuilder`], so the whole module requires the `delta`
//! feature. Run them explicitly once the environment stack is up and seeded:
//!
//! ```text
//! UC_ENDPOINT=http://localhost:8081/api/2.1/unity-catalog/ \
//! UC_TEST_TABLE=unity.default.numbers \
//! AWS_REGION=us-east-1 \
//! cargo test -p datafusion-unitycatalog --features delta --test unity_resolve -- --ignored --nocapture
//! ```
//!
//! Set `UC_TOKEN` for an authenticated server; omit it for a local
//! unauthenticated OSS server. Set `UC_TEST_TABLE_2` to a second table in the
//! *same bucket* to exercise the routing store's per-table credential
//! disambiguation.
#![cfg(feature = "delta")]

use std::sync::Arc;

use datafusion::catalog::AsyncCatalogProviderList;
use datafusion::common::TableReference;
use datafusion::prelude::SessionContext;
use datafusion_unitycatalog::catalog::{DeltaTableProviderBuilder, UnityCatalogProviderList};
use object_store::path::Path;
use unitycatalog_object_store::UnityObjectStoreFactory;

fn factory_from_env() -> Option<UnityObjectStoreFactoryFut> {
    let uri = std::env::var("UC_ENDPOINT").ok()?;
    Some(UnityObjectStoreFactoryFut { uri })
}

struct UnityObjectStoreFactoryFut {
    uri: String,
}

impl UnityObjectStoreFactoryFut {
    async fn build(self) -> UnityObjectStoreFactory {
        let mut builder = UnityObjectStoreFactory::builder().with_uri(self.uri);
        match std::env::var("UC_TOKEN") {
            Ok(token) => builder = builder.with_token(token),
            Err(_) => builder = builder.with_allow_unauthenticated(true),
        }
        if let Ok(region) = std::env::var("AWS_REGION") {
            builder = builder.with_aws_region(region);
        }
        builder.build().await.expect("failed to build UC factory")
    }
}

/// Resolve a UC table to a provider and scan it, asserting we get rows back.
#[tokio::test]
#[ignore = "requires a live Unity Catalog server (set UC_ENDPOINT)"]
async fn resolve_and_scan_uc_table() {
    let Some(factory_fut) = factory_from_env() else {
        eprintln!("UC_ENDPOINT not set; skipping");
        return;
    };
    let full_name = std::env::var("UC_TEST_TABLE").expect("set UC_TEST_TABLE=catalog.schema.table");

    let factory = Arc::new(factory_fut.build().await);
    let ctx = SessionContext::new();
    let builder = Arc::new(DeltaTableProviderBuilder::new(ctx.clone()));
    let resolver = UnityCatalogProviderList::new(factory, ctx.runtime_env(), builder);

    // Drive resolution exactly as the session does at plan time.
    let reference = TableReference::parse_str(&full_name);
    let config = ctx.copied_config();
    let resolved = resolver
        .resolve(std::slice::from_ref(&reference), &config)
        .await
        .expect("resolution failed");

    let catalog_name = reference.catalog().expect("table must be fully qualified");
    let catalog = resolved
        .catalog(catalog_name)
        .unwrap_or_else(|| panic!("catalog '{catalog_name}' not resolved"));
    let schema = catalog
        .schema(reference.schema().unwrap())
        .expect("schema not resolved");
    let provider = schema
        .table(reference.table())
        .await
        .expect("table lookup errored")
        .expect("table not resolved");

    // Register and scan via SQL to exercise the full read path.
    ctx.register_table(reference.clone(), provider)
        .expect("failed to register resolved provider");
    let df = ctx
        .sql(&format!("SELECT * FROM {full_name}"))
        .await
        .expect("planning failed");
    let batches = df.collect().await.expect("scan failed");
    let rows: usize = batches.iter().map(|b| b.num_rows()).sum();
    println!("scanned {rows} rows from {full_name}");
    assert!(rows > 0, "expected at least one row from {full_name}");
}

/// Smoke test the vended credential directly, before any DataFusion wiring, so
/// AWS credential-validation failures against the custom UC image surface here
/// rather than deep in a scan.
#[tokio::test]
#[ignore = "requires a live Unity Catalog server (set UC_ENDPOINT)"]
async fn vend_and_list_uc_table() {
    use futures::TryStreamExt;
    use unitycatalog_object_store::TableOperation;

    let Some(factory_fut) = factory_from_env() else {
        eprintln!("UC_ENDPOINT not set; skipping");
        return;
    };
    let full_name = std::env::var("UC_TEST_TABLE").expect("set UC_TEST_TABLE=catalog.schema.table");
    let factory = factory_fut.build().await;

    let store = factory
        .for_table(full_name.clone(), TableOperation::Read)
        .await
        .expect("vending credentials failed");
    let listing: Vec<_> = store
        .as_dyn()
        .list(Some(&Path::from("")))
        .try_collect()
        .await
        .expect("listing the table location failed (check AWS credential validation)");
    println!("listed {} objects under {full_name}", listing.len());
    assert!(!listing.is_empty(), "expected objects under {full_name}");
}
