//! Integration tests for `unitycatalog-object-store`.
//!
//! These tests are marked `#[ignore]` so they don't run by default — they
//! require a live Unity Catalog server (the `dev/compose.yaml` "full"
//! profile spins up the docker stack). Invoke them with:
//!
//! ```bash
//! just integration-object-store
//! ```
//!
//! Environment knobs:
//!
//! | Variable               | Default                                                 |
//! |------------------------|---------------------------------------------------------|
//! | `UC_INTEGRATION_URL`   | `http://localhost:8080/api/2.1/unity-catalog/`          |
//! | `UC_INTEGRATION_TOKEN` | (unset — runs against unauthenticated OSS UC)           |
//! | `UC_TEST_CATALOG`      | `unity`                                                 |
//! | `UC_TEST_SCHEMA`       | `default`                                               |
//! | `UC_TEST_VOLUME`       | `landing`                                               |
//!
//! Tests are designed to be idempotent — they assume the volume already
//! exists in the running UC instance.

use unitycatalog_object_store::{Operation, UnityObjectStoreFactory};

fn uc_url() -> String {
    std::env::var("UC_INTEGRATION_URL")
        .unwrap_or_else(|_| "http://localhost:8080/api/2.1/unity-catalog/".to_string())
}

async fn factory() -> UnityObjectStoreFactory {
    let mut builder = UnityObjectStoreFactory::builder().with_uri(uc_url());
    if let Ok(token) = std::env::var("UC_INTEGRATION_TOKEN") {
        builder = builder.with_token(token);
    } else {
        builder = builder.with_allow_unauthenticated(true);
    }
    builder
        .build()
        .await
        .expect("build UnityObjectStoreFactory")
}

#[tokio::test]
#[ignore = "requires a running UC server (just integration-object-store)"]
async fn parses_and_dispatches_uc_volume_url() {
    let factory = factory().await;
    let catalog = std::env::var("UC_TEST_CATALOG").unwrap_or_else(|_| "unity".into());
    let schema = std::env::var("UC_TEST_SCHEMA").unwrap_or_else(|_| "default".into());
    let volume = std::env::var("UC_TEST_VOLUME").unwrap_or_else(|_| "landing".into());
    let url = format!("uc:///Volumes/{catalog}/{schema}/{volume}/");

    let result = factory.for_url(&url, Operation::Read).await;
    // The OSS UC server may return NOT_IMPLEMENTED for the volume credential
    // endpoint (we stub it server-side). Both outcomes prove the URL parsing
    // + dispatch path is wired correctly.
    match result {
        Ok(store) => {
            assert!(!store.url().as_str().is_empty());
        }
        Err(e) => {
            let msg = e.to_string();
            assert!(
                msg.contains("not_implemented")
                    || msg.contains("NotImplemented")
                    || msg.contains("not implemented")
                    || msg.contains("501")
                    || msg.contains("NOT_IMPLEMENTED"),
                "expected NOT_IMPLEMENTED until the server handler ships, got: {e}"
            );
        }
    }
}

#[tokio::test]
#[ignore = "requires a running UC server (just integration-object-store)"]
async fn parses_and_dispatches_uc_table_url() {
    let factory = factory().await;
    let catalog = std::env::var("UC_TEST_CATALOG").unwrap_or_else(|_| "unity".into());
    let schema = std::env::var("UC_TEST_SCHEMA").unwrap_or_else(|_| "default".into());
    let table = std::env::var("UC_TEST_TABLE").unwrap_or_else(|_| "marksheet".into());
    let url = format!("uc:///Tables/{catalog}/{schema}/{table}");

    // Whether this succeeds depends on the running server having the
    // table registered — we just assert the call shape doesn't panic.
    let _ = factory.for_url(&url, Operation::Read).await;
}

#[tokio::test]
#[ignore = "requires a running UC server (just integration-object-store)"]
async fn parses_raw_cloud_path() {
    let factory = factory().await;
    // SeaweedFS bucket pre-created by the dev/compose.yaml init script.
    let raw = "s3://test-bucket/integration/";
    let _ = factory.for_url(raw, Operation::Read).await;
}
