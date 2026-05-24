//! Walkthroughs for `unitycatalog-object-store`.
//!
//! These functions also serve as snippets injected into the docs site —
//! see the `// [snippet:...] // [/snippet:...]` tags below.

use std::sync::Arc;

use unitycatalog_object_store::{Operation, PathOperation, UnityObjectStoreFactory};

// [snippet:object_store_factory]
/// Build the factory once per process and reuse it across requests.
///
/// In production, pull `base_url` and `token` from your secret store
/// instead of environment variables.
pub async fn build_factory() -> UnityObjectStoreFactory {
    let base_url = std::env::var("UC_BASE_URL")
        .unwrap_or_else(|_| "http://localhost:8080/api/2.1/unity-catalog/".into());
    let token: Option<String> = std::env::var("UC_TOKEN").ok();

    UnityObjectStoreFactory::builder()
        .with_uri(base_url)
        .with_token(token)
        .with_allow_unauthenticated(true) // local UC: development only
        .build()
        .await
        .expect("failed to build factory")
}
// [/snippet:object_store_factory]

// [snippet:object_store_for_url]
/// Address any Unity Catalog securable with a single `uc://` URL.
///
/// The factory routes:
///
/// * `uc:///Volumes/<c>/<s>/<v>[/<path>]` → `temporary-volume-credentials`
/// * `uc:///Tables/<c>/<s>/<t>`           → `temporary-table-credentials`
/// * `s3://`, `gs://`, `abfss://`, …      → `temporary-path-credentials`
pub async fn list_via_uc_url(factory: &UnityObjectStoreFactory) {
    let store = factory
        .for_url("uc:///Volumes/main/default/landing/raw/", Operation::Read)
        .await
        .expect("vend credentials");

    println!("listing {}", store.url());
    let entries = futures::StreamExt::collect::<Vec<_>>(store.as_dyn().list(None)).await;
    println!("found {} entries", entries.len());
}
// [/snippet:object_store_for_url]

// [snippet:object_store_for_table]
/// Read a Unity Catalog table's data files directly from its storage root.
pub async fn read_table_files(factory: &UnityObjectStoreFactory) {
    let store = factory
        .for_table("main.default.orders", Operation::Read.into())
        .await
        .expect("vend credentials");

    let entries = futures::StreamExt::collect::<Vec<_>>(store.as_dyn().list(None)).await;
    println!("table data files: {}", entries.len());
}
// [/snippet:object_store_for_table]

// [snippet:object_store_for_volume]
/// Walk a Unity Catalog volume with the prefixed default store.
///
/// `store.as_dyn()` is rooted at the volume's storage location; pass
/// volume-relative paths to `list` / `get` / `put`.
pub async fn list_volume_contents(factory: &UnityObjectStoreFactory) {
    let store = factory
        .for_volume("main.default.landing", Operation::Read.into())
        .await
        .expect("vend credentials");

    let entries = futures::StreamExt::collect::<Vec<_>>(store.as_dyn().list(None)).await;
    println!("volume entries: {}", entries.len());
}
// [/snippet:object_store_for_volume]

// [snippet:object_store_for_path]
/// Fall back to `temporary-path-credentials` for raw cloud URLs.
pub async fn read_raw_cloud_path(factory: &UnityObjectStoreFactory) {
    let url = url::Url::parse("s3://example-bucket/path/").unwrap();
    let store = factory
        .for_path(&url, PathOperation::Read)
        .await
        .expect("vend credentials");

    let entries = futures::StreamExt::collect::<Vec<_>>(store.as_dyn().list(None)).await;
    println!("path entries: {}", entries.len());
}
// [/snippet:object_store_for_path]

// [snippet:object_store_datafusion]
/// Plug a Unity Catalog-backed store into DataFusion's `RuntimeEnv` so
/// SQL queries that reference `s3://`/`gs://`/`abfss://` URLs picked from
/// the vended credential transparently work.
///
/// This example is intentionally framework-light — it does not depend on
/// DataFusion in this crate. Pseudocode follows:
///
/// ```ignore
/// use datafusion::execution::runtime_env::RuntimeEnv;
///
/// let store = factory.for_table("main.default.orders", Operation::Read.into()).await?;
/// let runtime = RuntimeEnv::default();
/// runtime
///     .object_store_registry
///     .register_store(store.url(), store.as_dyn());
/// // DataFusion will now resolve `store.url()` through the UC-backed store.
/// ```
pub async fn datafusion_registration(_factory: &UnityObjectStoreFactory) {
    // Intentional placeholder — see the docstring above for the
    // recommended DataFusion wiring.
    let _: Arc<dyn object_store::ObjectStore> = Arc::new(object_store::memory::InMemory::new());
}
// [/snippet:object_store_datafusion]
