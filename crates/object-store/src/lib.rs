//! `object_store` integration for Unity Catalog.
//!
//! This crate adapts Unity Catalog's credential-vending APIs to the
//! [`object_store`](https://docs.rs/object_store) trait, so any framework
//! that accepts an `Arc<dyn ObjectStore>` (DataFusion, `delta_kernel`,
//! `parquet`, …) can read and write data governed by Unity Catalog
//! volumes, tables, or external locations with no extra glue.
//!
//! # Quickstart
//!
//! ```no_run
//! use unitycatalog_object_store::{Operation, UnityObjectStoreFactory};
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let factory = UnityObjectStoreFactory::builder()
//!     .with_uri("https://my-workspace.cloud.databricks.com/api/2.1/unity-catalog/")
//!     .with_token(std::env::var("DATABRICKS_TOKEN").unwrap())
//!     .build()
//!     .await?;
//!
//! // Address a UC securable directly with a `uc://` URL …
//! let store = factory
//!     .for_url("uc:///Volumes/main/default/landing/raw/", Operation::Read)
//!     .await?;
//!
//! // … and use it like any other `object_store`.
//! let listing = futures::StreamExt::collect::<Vec<_>>(store.as_dyn().list(None)).await;
//! # Ok(()) }
//! ```
//!
//! # URL scheme
//!
//! See [`UCReference`] for the full grammar. In short:
//!
//! - `uc:///Volumes/<catalog>/<schema>/<volume>[/<path>]`
//! - `uc:///Tables/<catalog>/<schema>/<table>`
//! - `s3://`, `gs://`, `abfss://`, `r2://`, … — raw cloud URLs, vended via
//!   `temporary-path-credentials`.
//!
//! The kind segment is **case-insensitive** (so `/Volumes/`, `/volumes/`,
//! and `/VOLUMES/` all work); the capitalised form is canonical because it
//! mirrors the Databricks workspace POSIX path convention. The Databricks
//! `vol+dbfs:/Volumes/...` alias is also accepted.

use std::sync::Arc;

use object_store::aws::AmazonS3Builder;
use object_store::azure::MicrosoftAzureBuilder;
use object_store::gcp::GoogleCloudStorageBuilder;
use object_store::path::Path;
use object_store::prefix::PrefixStore;
use object_store::{ObjectStore, Result};
use olai_http::CloudClient;
use unitycatalog_client::{TemporaryCredentialClient, UnityCatalogClient};
use unitycatalog_common::temporary_credentials::v1::TemporaryCredential;
use url::Url;

use crate::credential::{
    SecurableRef, as_aws, as_azure, as_gcp, aws_access_point, new_aws, new_azure, new_gcp,
};
pub use crate::error::Error;
pub use unitycatalog_common::UCReference;
// Re-export the reference / operation enums so consumers do not need a direct
// dependency on `unitycatalog-client` for the common case.
pub use unitycatalog_client::{
    PathOperation, TableOperation, TableReference, VolumeOperation, VolumeReference,
};

mod credential;
mod error;
/// Builder for [`UnityObjectStoreFactory`].
#[derive(Debug, Clone, Default)]
pub struct UnityObjectStoreFactoryBuilder {
    /// Base URL of the Unity Catalog REST API
    /// (e.g. `https://<workspace>.cloud.databricks.com/api/2.1/unity-catalog/`).
    uri: Option<String>,
    /// Bearer token used for authentication.
    token: Option<String>,
    /// Permit construction without a token. Useful for local development
    /// against an unauthenticated OSS server; do not use in production.
    allow_unauthenticated: bool,
    /// Optional AWS region hint. Required when the data lives in a region
    /// other than `us-east-1` and the server does not return region info
    /// alongside the vended credential.
    aws_region: Option<String>,
}

impl UnityObjectStoreFactoryBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the URI of the Unity Catalog API
    /// (e.g. `https://<workspace>/api/2.1/unity-catalog/`).
    pub fn with_uri(mut self, uri: impl Into<String>) -> Self {
        self.uri = Some(uri.into());
        self
    }

    /// Set the [access token] used for bearer authentication.
    ///
    /// Accepts both `String` and `Option<String>` — pass `None` to clear
    /// a previously-set token (e.g. when reusing the builder).
    ///
    /// [access token]: https://docs.databricks.com/aws/en/dev-tools/auth/pat
    pub fn with_token(mut self, token: impl Into<Option<String>>) -> Self {
        self.token = token.into();
        self
    }

    /// Allow construction without any authentication credentials.
    ///
    /// Only intended for local development against an unauthenticated OSS
    /// Unity Catalog server — there should not be any unauthenticated UC
    /// servers in production deployments.
    pub fn with_allow_unauthenticated(mut self, allow_unauthenticated: bool) -> Self {
        self.allow_unauthenticated = allow_unauthenticated;
        self
    }

    /// Override the AWS region used for vended AWS credentials.
    ///
    /// When unset the factory falls back to (in order):
    /// 1. The `AWS_REGION` environment variable.
    /// 2. The `object_store` default region (`us-east-1`).
    ///
    /// This is a stop-gap until the server reliably returns region info
    /// alongside the credential.
    pub fn with_aws_region(mut self, aws_region: impl Into<Option<String>>) -> Self {
        self.aws_region = aws_region.into();
        self
    }

    pub async fn build(self) -> Result<UnityObjectStoreFactory> {
        let url = if let Some(uri) = self.uri {
            url::Url::parse(&uri).map_err(Error::from)?
        } else {
            return Err(Error::invalid_config("missing `uri` for Unity Catalog endpoint").into());
        };

        let cloud_client = if let Some(token) = self.token {
            CloudClient::new_with_token(token)
        } else if self.allow_unauthenticated {
            CloudClient::new_unauthenticated()
        } else {
            return Err(Error::invalid_config(
                "no token and `allow_unauthenticated` not set: cannot build credential client",
            )
            .into());
        };

        let creds = TemporaryCredentialClient::new_with_url(cloud_client.clone(), url.clone());
        let uc = UnityCatalogClient::new(cloud_client, url);
        Ok(UnityObjectStoreFactory {
            creds,
            uc,
            aws_region: self.aws_region,
        })
    }
}

/// A configured Unity Catalog `ObjectStore` ready for use.
///
/// The default [`Self::as_dyn`] returns a store that is automatically
/// prefixed to the credential-scoped sub-path (e.g. just the volume's
/// storage root); paths passed to `list`/`get`/`put` are interpreted
/// relative to that prefix. The unprefixed [`Self::root`] is an escape
/// hatch for callers that need to work at the bucket level.
#[derive(Clone)]
pub struct UCStore {
    /// Bucket-rooted store (credentials may be scoped to a sub-path).
    root: Arc<dyn ObjectStore>,
    /// The full cloud URL of the credential-scoped root.
    url: Url,
    /// Path within `root` the credential is scoped to.
    path: Path,
}

impl UCStore {
    /// Returns the credential-scoped store (prefixed at [`Self::prefix`]).
    ///
    /// This is the common case: callers list / read / write paths inside
    /// the volume or table the credential was vended for.
    pub fn as_dyn(&self) -> Arc<dyn ObjectStore> {
        if self.path.as_ref().is_empty() {
            self.root.clone()
        } else {
            Arc::new(PrefixStore::new(self.root.clone(), self.path.clone()))
        }
    }

    /// Returns the bucket-rooted store.
    ///
    /// The vended credential may not authorise access to siblings of
    /// [`Self::prefix`]; callers using `root()` are responsible for not
    /// accessing paths outside the scoped region.
    pub fn root(&self) -> Arc<dyn ObjectStore> {
        self.root.clone()
    }

    /// The full cloud URL of the credential-scoped root
    /// (e.g. `s3://bucket/prefix/inside/volume/`).
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// The prefix inside [`Self::root`] the credential is scoped to.
    pub fn prefix(&self) -> &Path {
        &self.path
    }
}

/// Factory that mints `object_store` instances backed by Unity Catalog
/// credential vending.
#[derive(Clone)]
pub struct UnityObjectStoreFactory {
    creds: TemporaryCredentialClient,
    uc: UnityCatalogClient,
    aws_region: Option<String>,
}

impl UnityObjectStoreFactory {
    pub fn builder() -> UnityObjectStoreFactoryBuilder {
        UnityObjectStoreFactoryBuilder::default()
    }

    /// Borrow the underlying [`UnityCatalogClient`] for catalog metadata
    /// operations (listing volumes, resolving table names, …).
    pub fn unity_client(&self) -> &UnityCatalogClient {
        &self.uc
    }

    /// Borrow the underlying credential-vending client. Most users want
    /// [`for_url`](Self::for_url) / [`for_volume`](Self::for_volume) /
    /// [`for_table`](Self::for_table) / [`for_path`](Self::for_path) instead.
    pub fn credentials_client(&self) -> &TemporaryCredentialClient {
        &self.creds
    }

    /// Build an [`UCStore`] for any supported URL.
    ///
    /// See [`UCReference`] for the supported URL grammar. Raw cloud URLs
    /// (`s3://`, `gs://`, `abfss://`, …) are routed to
    /// [`for_path`](Self::for_path).
    pub async fn for_url(&self, url: &str, op: Operation) -> Result<UCStore> {
        let reference = UCReference::parse(url)
            .map_err(crate::error::Error::from)
            .map_err(object_store::Error::from)?;
        match reference {
            UCReference::Volume {
                catalog,
                schema,
                volume,
                path,
            } => {
                let name = format!("{catalog}.{schema}.{volume}");
                let store = self.for_volume(name, op.into_volume()).await?;
                if path.is_empty() {
                    Ok(store)
                } else {
                    Ok(extend_prefix(store, &path))
                }
            }
            UCReference::Table {
                catalog,
                schema,
                table,
            } => {
                let name = format!("{catalog}.{schema}.{table}");
                self.for_table(name, op.into_table()).await
            }
            UCReference::Path(url) => self.for_path(&url, op.into_path()).await,
        }
    }

    /// Vend credentials for a table and return a prefixed store rooted at
    /// the table's storage location.
    ///
    /// The `table` argument accepts a `Uuid`, a [`String`] / `&str`
    /// containing a three-level `<catalog>.<schema>.<table>` name, or any
    /// [`TableReference`].
    pub async fn for_table(
        &self,
        table: impl Into<TableReference>,
        operation: TableOperation,
    ) -> Result<UCStore> {
        let (credential, table_id) = self
            .creds
            .temporary_table_credential(table, operation)
            .await
            .map_err(Error::from)?;
        let securable = SecurableRef::Table(table_id, operation);
        self.build_store(credential, securable).await
    }

    /// Vend credentials for a volume and return a prefixed store rooted at
    /// the volume's storage location.
    pub async fn for_volume(
        &self,
        volume: impl Into<VolumeReference>,
        operation: VolumeOperation,
    ) -> Result<UCStore> {
        let (credential, volume_id) = self
            .creds
            .temporary_volume_credential(volume, operation)
            .await
            .map_err(Error::from)?;
        let securable = SecurableRef::Volume(volume_id, operation);
        self.build_store(credential, securable).await
    }

    /// Vend credentials for a raw cloud URL (`s3://`, `gs://`, `abfss://`,
    /// …). Uses `temporary-path-credentials` under the hood.
    pub async fn for_path(&self, path: &Url, operation: PathOperation) -> Result<UCStore> {
        let (credential, _resolved) = self
            .creds
            .temporary_path_credential(path.clone(), operation, false)
            .await
            .map_err(Error::from)?;
        let securable = SecurableRef::Path(path.clone(), operation, Some(false));
        self.build_store(credential, securable).await
    }

    /// Vend credentials for a raw cloud URL with `dry_run` set to true.
    /// The server validates that credentials *could* be issued but the
    /// returned token is not usable for IO; useful for permission probes.
    pub async fn dry_run_path(&self, path: &Url, operation: PathOperation) -> Result<UCStore> {
        let (credential, _resolved) = self
            .creds
            .temporary_path_credential(path.clone(), operation, true)
            .await
            .map_err(Error::from)?;
        let securable = SecurableRef::Path(path.clone(), operation, Some(true));
        self.build_store(credential, securable).await
    }

    async fn build_store(
        &self,
        credential: TemporaryCredential,
        securable: SecurableRef,
    ) -> Result<UCStore> {
        let url = Url::parse(&credential.url).map_err(Error::from)?;
        let path = Path::from_url_path(url.path())?;
        let store = self.to_store(credential, securable).await?;
        Ok(UCStore {
            root: store,
            url,
            path,
        })
    }

    async fn to_store(
        &self,
        credential: TemporaryCredential,
        securable: SecurableRef,
    ) -> Result<Arc<dyn ObjectStore>> {
        if as_azure(&credential).is_ok() {
            let provider = new_azure(self.creds.clone(), &credential, securable).await?;
            let url = Url::parse(&credential.url).map_err(Error::from)?;
            let store = MicrosoftAzureBuilder::new()
                .with_url(url.to_string())
                .with_credentials(Arc::new(provider))
                .build()?;
            return Ok(Arc::new(store));
        }

        if as_aws(&credential).is_ok() {
            let access_point = aws_access_point(&credential);
            let provider = new_aws(self.creds.clone(), &credential, securable).await?;
            let url = Url::parse(&credential.url).map_err(Error::from)?;
            let mut builder = AmazonS3Builder::new()
                .with_url(url.to_string())
                .with_credentials(Arc::new(provider));
            // Prefer an explicit override; otherwise honour `AWS_REGION`
            // before falling back to the object_store default.
            if let Some(region) = self
                .aws_region
                .clone()
                .or_else(|| std::env::var("AWS_REGION").ok())
            {
                builder = builder.with_region(region);
            }
            // Where the server returns an S3 access-point ARN, use it as the
            // bucket so SigV4 signatures match what STS authorised.
            if let Some(ap) = access_point {
                builder = builder.with_bucket_name(ap);
            }
            let store = builder.build()?;
            return Ok(Arc::new(store));
        }

        if as_gcp(&credential).is_ok() {
            let provider = new_gcp(self.creds.clone(), &credential, securable).await?;
            let url = Url::parse(&credential.url).map_err(Error::from)?;
            let store = GoogleCloudStorageBuilder::new()
                .with_url(url.to_string())
                .with_credentials(Arc::new(provider))
                .build()?;
            return Ok(Arc::new(store));
        }

        Err(
            Error::InvalidCredential("Failed to match credential with storage type".to_string())
                .into(),
        )
    }
}

/// Unified read/write operation used by [`UnityObjectStoreFactory::for_url`].
///
/// The factory translates this to the operation enum expected by each
/// vending endpoint:
///
/// | `Operation`  | Volume        | Table       | Path             |
/// |--------------|---------------|-------------|------------------|
/// | `Read`       | `READ_VOLUME` | `READ`      | `PATH_READ`      |
/// | `ReadWrite`  | `WRITE_VOLUME`| `READ_WRITE`| `PATH_READ_WRITE`|
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Read,
    ReadWrite,
}

impl Operation {
    fn into_volume(self) -> VolumeOperation {
        match self {
            Operation::Read => VolumeOperation::Read,
            Operation::ReadWrite => VolumeOperation::ReadWrite,
        }
    }

    fn into_table(self) -> TableOperation {
        match self {
            Operation::Read => TableOperation::Read,
            Operation::ReadWrite => TableOperation::ReadWrite,
        }
    }

    fn into_path(self) -> PathOperation {
        match self {
            Operation::Read => PathOperation::Read,
            Operation::ReadWrite => PathOperation::ReadWrite,
        }
    }
}

impl From<Operation> for TableOperation {
    fn from(op: Operation) -> Self {
        op.into_table()
    }
}

impl From<Operation> for VolumeOperation {
    fn from(op: Operation) -> Self {
        op.into_volume()
    }
}

impl From<Operation> for PathOperation {
    fn from(op: Operation) -> Self {
        op.into_path()
    }
}

/// Returns a [`UCStore`] whose prefix is `store.prefix() + extra`, leaving
/// the underlying bucket-rooted store untouched.
fn extend_prefix(store: UCStore, extra: &str) -> UCStore {
    let mut url = store.url.clone();
    // Append the extra path component(s), keeping the trailing slash.
    {
        let mut segs = url.path_segments_mut().expect("cloud URL has a path");
        segs.pop_if_empty();
        for part in extra.split('/').filter(|p| !p.is_empty()) {
            segs.push(part);
        }
    }
    let new_path = if store.path.as_ref().is_empty() {
        Path::from(extra)
    } else {
        let base = store.path.as_ref().trim_end_matches('/');
        let extra = extra.trim_start_matches('/');
        Path::from(format!("{base}/{extra}"))
    };
    UCStore {
        root: store.root,
        url,
        path: new_path,
    }
}

#[cfg(test)]
mod tests {
    use futures::TryStreamExt;

    use super::*;

    /// Live test against a Databricks workspace. Requires
    /// `DATABRICKS_HOST` + `DATABRICKS_TOKEN` to be set. Marked `#[ignore]`
    /// because CI shouldn't hit a real workspace.
    #[tokio::test]
    #[ignore]
    async fn list_store_via_temp_credential() {
        let databricks_host = std::env::var("DATABRICKS_HOST").unwrap();
        let databricks_token = std::env::var("DATABRICKS_TOKEN").unwrap();
        let factory = UnityObjectStoreFactory::builder()
            .with_uri(format!("{databricks_host}/api/2.1/unity-catalog/"))
            .with_token(databricks_token)
            .with_aws_region("eu-north-1".to_string())
            .build()
            .await
            .unwrap();

        let volume_path = url::Url::parse("s3://open-lakehouse-dev/volumes/").unwrap();
        let store = factory
            .for_path(&volume_path, PathOperation::Read)
            .await
            .unwrap();
        let files: Vec<_> = store.as_dyn().list(None).try_collect().await.unwrap();
        println!("files: {files:?}");
    }
}
