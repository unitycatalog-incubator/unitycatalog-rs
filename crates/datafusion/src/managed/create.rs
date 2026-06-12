//! Create a Unity Catalog catalog-managed Delta table end-to-end.
//!
//! Follows the kernel's documented recipe (`delta-kernel-unity-catalog/src/utils/create_table.rs`),
//! wired to the unitycatalog-rs [`DeltaV1Client`]:
//!
//! 1. `createStagingTable` → UC allocates a `table_id` + managed `staging_location`.
//! 2. Build a credentialed object store from the staging response, and a kernel engine over it.
//! 3. `kernel::create_table(...).build(engine, UnityCatalogCommitter).commit(engine)` writes
//!    `_delta_log/0.json` (the committer writes it directly to the published path; the kernel
//!    auto-enables `inCommitTimestamp` for `catalogManaged`).
//! 4. Load the v0 snapshot, derive the UC-registration properties, and `createTable` to finalize
//!    the table in Unity Catalog.

use std::collections::HashMap;
use std::sync::Arc;

use datafusion::arrow::datatypes::{DataType as ArrowDataType, SchemaRef as ArrowSchemaRef};
use delta_kernel::Engine;
use delta_kernel::engine::arrow_conversion::TryIntoKernel;
use delta_kernel::engine::default::DefaultEngineBuilder;
use delta_kernel::schema::{SchemaRef, StructType};
use delta_kernel::snapshot::Snapshot;
use delta_kernel::transaction::CommitResult;
use delta_kernel::transaction::create_table::create_table;
use delta_kernel::transaction::data_layout::DataLayout;
use object_store::ObjectStore;
use object_store::aws::AmazonS3Builder;
use unitycatalog_client::DeltaV1Client;
use unitycatalog_common::models::delta::v1::{
    DeltaCreateStagingTableRequest, DeltaCreateTableRequest, DeltaDataSourceFormat, DeltaDataType,
    DeltaStagingTableResponse, DeltaStructField, DeltaStructType, DeltaTableType,
};
use url::Url;

use super::committer::UnityCatalogCommitter;

// UC catalog-managed contract identifiers (mirror the kernel's `constants`).
const CATALOG_MANAGED_FEATURE_KEY: &str = "delta.feature.catalogManaged";
const VACUUM_PROTOCOL_CHECK_FEATURE_KEY: &str = "delta.feature.vacuumProtocolCheck";
// The live Java UC OSS server additionally requires the `v2Checkpoint` reader feature on a
// MANAGED table (the kernel's create_table allow-lists it). The fork's helper omits it; we
// enable it + its checkpoint-policy property to satisfy the server's contract.
const V2_CHECKPOINT_FEATURE_KEY: &str = "delta.feature.v2Checkpoint";
const CHECKPOINT_POLICY_KEY: &str = "delta.checkpointPolicy";
const CHECKPOINT_POLICY_V2: &str = "v2";
const FEATURE_SUPPORTED: &str = "supported";
const UC_TABLE_ID_KEY: &str = "io.unitycatalog.tableId";
const METASTORE_LAST_UPDATE_VERSION: &str = "delta.lastUpdateVersion";
const METASTORE_LAST_COMMIT_TIMESTAMP: &str = "delta.lastCommitTimestamp";

/// Error type for the managed-table connector.
#[derive(Debug, thiserror::Error)]
pub enum CreateManagedTableError {
    #[error("unity catalog client error: {0}")]
    Client(#[from] unitycatalog_client::Error),
    #[error("delta kernel error: {0}")]
    Kernel(#[from] delta_kernel::Error),
    #[error("object store error: {0}")]
    ObjectStore(#[from] object_store::Error),
    #[error("{0}")]
    Other(String),
}

impl CreateManagedTableError {
    pub(crate) fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }
}

/// A handle to a freshly created Unity Catalog managed table.
#[derive(Debug, Clone)]
pub struct ManagedTable {
    /// The UC-allocated table id (`io.unitycatalog.tableId`).
    pub table_id: String,
    /// The managed storage location UC allocated for the table.
    pub location: Url,
}

/// Create a Unity Catalog managed Delta table `catalog.schema.table` with the given Arrow schema
/// (and optional identity partition columns), committing version 0 through the kernel committer
/// and finalizing the table in UC.
///
/// `engine_info` is a free-form engine identifier recorded in the commit (e.g. `"unitycatalog-rs/0.1"`).
pub async fn create_managed_table(
    client: Arc<DeltaV1Client>,
    catalog: &str,
    schema_name: &str,
    table: &str,
    arrow_schema: ArrowSchemaRef,
    partition_columns: Vec<String>,
    engine_info: &str,
) -> Result<ManagedTable, CreateManagedTableError> {
    // 1. Reserve a staging table: UC allocates the id + managed location and advertises the contract.
    let staging = client
        .create_staging_table(
            catalog,
            schema_name,
            &DeltaCreateStagingTableRequest {
                name: table.to_string(),
            },
        )
        .await?;
    let table_id = staging.table_id.clone();
    let location = Url::parse(&ensure_trailing_slash(&staging.location))
        .map_err(|e| CreateManagedTableError::other(format!("invalid staging location: {e}")))?;

    // 2. Credentialed object store (from the staging response) + a kernel engine over it.
    let store = build_staging_store(&staging, &location)?;
    let engine = DefaultEngineBuilder::new(store).build();

    // 3. Write `0.json` via kernel create_table + our catalog committer.
    let kernel_schema: SchemaRef = arrow_to_kernel_schema(&arrow_schema)?;
    let committer = UnityCatalogCommitter::new(
        client.clone(),
        catalog,
        schema_name,
        table,
        table_id.clone(),
    );

    let mut builder = create_table(location.as_str(), kernel_schema, engine_info)
        .with_table_properties(get_required_properties_for_disk(&table_id));
    if !partition_columns.is_empty() {
        builder = builder.with_data_layout(DataLayout::partitioned(partition_columns.clone()));
    }
    let committed = match builder
        .build(&engine, Box::new(committer))?
        .commit(&engine)?
    {
        CommitResult::CommittedTransaction(c) => c,
        CommitResult::ConflictedTransaction(_) => {
            return Err(CreateManagedTableError::other(
                "version 0 already exists at the staging location",
            ));
        }
        CommitResult::RetryableTransaction(_) => {
            return Err(CreateManagedTableError::other(
                "retryable error writing version 0 commit",
            ));
        }
    };

    // 4. Derive UC-registration properties from the v0 snapshot, then finalize the table in UC.
    //    Prefer the post-commit snapshot; fall back to a fresh load (the committer published
    //    `0.json` directly, so a plain snapshot load reads v0).
    let snapshot = match committed.post_commit_snapshot() {
        Some(s) => s.clone(),
        None => Snapshot::builder_for(location.as_str()).build(&engine)?,
    };
    let uc_properties = get_final_required_properties_for_uc(&snapshot, &engine)?;

    client
        .create_table(
            catalog,
            schema_name,
            &DeltaCreateTableRequest {
                name: table.to_string(),
                location: location.to_string(),
                table_type: DeltaTableType::Managed,
                data_source_format: Some(DeltaDataSourceFormat::Delta),
                comment: None,
                columns: arrow_to_delta_columns(&arrow_schema)?,
                partition_columns: (!partition_columns.is_empty()).then_some(partition_columns),
                protocol: snapshot_protocol(&snapshot),
                properties: uc_properties.into_iter().collect(),
                domain_metadata: None,
                last_commit_timestamp_ms: snapshot.get_in_commit_timestamp(&engine)?.ok_or_else(
                    || CreateManagedTableError::other("v0 snapshot has no in-commit timestamp"),
                )?,
                uniform: None,
            },
        )
        .await?;

    Ok(ManagedTable { table_id, location })
}

/// Table properties that must be written to disk (in `0.json`) for a UC catalog-managed table.
/// (ICT enablement is added automatically by the kernel's create_table when `catalogManaged` is set.)
pub fn get_required_properties_for_disk(uc_table_id: &str) -> HashMap<String, String> {
    // Enable the required features via feature signals. `delta.checkpointPolicy` is NOT set
    // here — the kernel rejects it as a create-time property; enabling the `v2Checkpoint`
    // feature is sufficient on disk, and the server-facing `delta.checkpointPolicy=v2` is
    // derived from the committed protocol in `get_final_required_properties_for_uc`.
    [
        (CATALOG_MANAGED_FEATURE_KEY, FEATURE_SUPPORTED),
        (VACUUM_PROTOCOL_CHECK_FEATURE_KEY, FEATURE_SUPPORTED),
        (V2_CHECKPOINT_FEATURE_KEY, FEATURE_SUPPORTED),
        (UC_TABLE_ID_KEY, uc_table_id),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect()
}

/// Properties to send to UC when finalizing the table — derived from the post-commit v0 snapshot.
/// Ported from the kernel's `get_final_required_properties_for_uc` (uses `internal-api` accessors).
pub fn get_final_required_properties_for_uc(
    snapshot: &Snapshot,
    engine: &dyn Engine,
) -> Result<HashMap<String, String>, CreateManagedTableError> {
    if snapshot.version() != 0 {
        return Err(CreateManagedTableError::other(format!(
            "expected a version 0 snapshot, got version {}",
            snapshot.version()
        )));
    }
    let mut properties = snapshot.metadata_configuration().clone();
    properties.extend(snapshot.get_protocol_derived_properties());
    // The server's MANAGED contract wants `delta.checkpointPolicy=v2` explicitly; it follows
    // from the `v2Checkpoint` feature but isn't emitted by the protocol-derived properties.
    properties.insert(
        CHECKPOINT_POLICY_KEY.to_string(),
        CHECKPOINT_POLICY_V2.to_string(),
    );
    properties.insert(
        METASTORE_LAST_UPDATE_VERSION.to_string(),
        snapshot.version().to_string(),
    );
    let ts = snapshot
        .get_in_commit_timestamp(engine)?
        .ok_or_else(|| CreateManagedTableError::other("v0 snapshot has no in-commit timestamp"))?;
    properties.insert(METASTORE_LAST_COMMIT_TIMESTAMP.to_string(), ts.to_string());
    Ok(properties)
}

// =======================================================================
// Helpers
// =======================================================================

fn arrow_to_kernel_schema(arrow: &ArrowSchemaRef) -> Result<SchemaRef, CreateManagedTableError> {
    let st: StructType = arrow
        .as_ref()
        .try_into_kernel()
        .map_err(|e| CreateManagedTableError::other(format!("arrow→kernel schema: {e}")))?;
    Ok(Arc::new(st))
}

/// Build the UC `columns` payload (Delta schema struct) from the Arrow schema. UC's
/// `DeltaDataType::Primitive` strings are Delta type names; map the common primitives.
fn arrow_to_delta_columns(
    arrow: &ArrowSchemaRef,
) -> Result<DeltaStructType, CreateManagedTableError> {
    let fields = arrow
        .fields()
        .iter()
        .map(|f| {
            Ok(DeltaStructField {
                name: f.name().clone(),
                data_type: DeltaDataType::Primitive(arrow_primitive_to_delta(f.data_type())?),
                nullable: f.is_nullable(),
                metadata: Default::default(),
            })
        })
        .collect::<Result<Vec<_>, CreateManagedTableError>>()?;
    Ok(DeltaStructType {
        type_tag: Default::default(),
        fields,
    })
}

/// Map an Arrow primitive to its Delta type name (`long`, `string`, …). Errors on unsupported /
/// nested types — the connector targets flat schemas for the initial implementation.
fn arrow_primitive_to_delta(dt: &ArrowDataType) -> Result<String, CreateManagedTableError> {
    let name = match dt {
        ArrowDataType::Boolean => "boolean",
        ArrowDataType::Int8 => "byte",
        ArrowDataType::Int16 => "short",
        ArrowDataType::Int32 => "integer",
        ArrowDataType::Int64 => "long",
        ArrowDataType::Float32 => "float",
        ArrowDataType::Float64 => "double",
        ArrowDataType::Utf8 | ArrowDataType::LargeUtf8 => "string",
        ArrowDataType::Binary | ArrowDataType::LargeBinary => "binary",
        ArrowDataType::Date32 => "date",
        ArrowDataType::Timestamp(_, _) => "timestamp",
        other => {
            return Err(CreateManagedTableError::other(format!(
                "unsupported column type for managed-table create: {other:?}"
            )));
        }
    };
    Ok(name.to_string())
}

/// The protocol to send in the UC createTable request, read off the committed v0 snapshot so it
/// matches exactly what was written to `0.json`.
fn snapshot_protocol(snapshot: &Snapshot) -> unitycatalog_common::models::delta::v1::DeltaProtocol {
    use unitycatalog_common::models::delta::v1::DeltaProtocol;
    let p = snapshot.table_configuration().protocol();
    DeltaProtocol {
        min_reader_version: p.min_reader_version(),
        min_writer_version: p.min_writer_version(),
        reader_features: Some(
            p.reader_features()
                .iter()
                .flat_map(|fs| fs.iter().map(|f| f.to_string()))
                .collect(),
        ),
        writer_features: Some(
            p.writer_features()
                .iter()
                .flat_map(|fs| fs.iter().map(|f| f.to_string()))
                .collect(),
        ),
    }
}

fn ensure_trailing_slash(s: &str) -> String {
    if s.ends_with('/') {
        s.to_string()
    } else {
        format!("{s}/")
    }
}

/// Build a bucket-rooted S3 object store from the credentials the `createStagingTable` response
/// returned. (Vending by table name 404s during staging — the table isn't real yet — so we use
/// the staging response's own `storage_credentials`.)
pub(crate) fn build_staging_store(
    staging: &DeltaStagingTableResponse,
    location: &Url,
) -> Result<Arc<dyn ObjectStore>, CreateManagedTableError> {
    let cred = staging
        .storage_credentials
        .iter()
        .find(|c| {
            location
                .as_str()
                .starts_with(c.prefix.trim_end_matches('/'))
        })
        .or_else(|| staging.storage_credentials.first())
        .ok_or_else(|| {
            CreateManagedTableError::other("staging response carried no storage_credentials")
        })?;
    let cfg = &cred.config;
    let bucket = location
        .host_str()
        .ok_or_else(|| CreateManagedTableError::other("staging location has no bucket/host"))?;

    let mut builder = AmazonS3Builder::new()
        .with_bucket_name(bucket)
        .with_access_key_id(cfg.s3_access_key_id.clone().ok_or_else(|| {
            CreateManagedTableError::other("staging credential missing s3.access-key-id")
        })?)
        .with_secret_access_key(cfg.s3_secret_access_key.clone().ok_or_else(|| {
            CreateManagedTableError::other("staging credential missing s3.secret-access-key")
        })?);
    if let Some(token) = cfg.s3_session_token.clone() {
        builder = builder.with_token(token);
    }
    if let Ok(endpoint) = std::env::var("AWS_ENDPOINT_URL") {
        builder = builder.with_endpoint(endpoint).with_allow_http(true);
    }
    builder =
        builder.with_region(std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".into()));
    Ok(Arc::new(builder.build()?))
}
