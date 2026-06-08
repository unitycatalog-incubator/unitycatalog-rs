//! Hand-written client for the UC Delta REST API (`/delta/v1/...`).
//!
//! The Delta API is a standalone REST protocol (not a generated resource API),
//! so — like [`crate::temporary_credentials`] — it is hand-maintained. The wire
//! DTOs are shared with the server via
//! [`unitycatalog_common::models::delta::v1`]. This client covers the
//! catalog-managed table lifecycle needed to create and commit a table:
//! `createStagingTable`, `createTable`, `loadTable`, and `updateTable` (the
//! `add-commit` / `set-latest-backfilled-version` flow).

use olai_http::CloudClient;
use unitycatalog_common::models::delta::v1::{
    DeltaCreateStagingTableRequest, DeltaCreateTableRequest, DeltaLoadTableResponse,
    DeltaStagingTableResponse, DeltaUpdateTableRequest,
};
use url::Url;

use crate::Result;

/// Client for the `/delta/v1/` Delta REST API.
///
/// Constructed from a [`CloudClient`] (carrying auth) and the Unity Catalog base
/// URL. Prefer [`UnityCatalogClient::delta_v1`](crate::UnityCatalogClient::delta_v1).
#[derive(Clone)]
pub struct DeltaV1Client {
    client: CloudClient,
    base_url: Url,
}

impl DeltaV1Client {
    /// Create a new Delta v1 client. `base_url` is normalized to end in `/` so it
    /// joins cleanly with the relative endpoint paths.
    pub fn new(client: CloudClient, mut base_url: Url) -> Self {
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        Self { client, base_url }
    }

    /// Create a staging table — `POST /delta/v1/catalogs/{catalog}/schemas/{schema}/staging-tables`.
    ///
    /// Allocates an immutable table id and a managed storage `location`, and
    /// advertises the catalog-managed contract (required/suggested protocol +
    /// properties) plus `READ_WRITE` credentials for writing the initial
    /// `_delta_log/0.json`. The catalog/schema are taken from the path; the
    /// request body carries only the table name.
    pub async fn create_staging_table(
        &self,
        catalog: &str,
        schema: &str,
        request: &DeltaCreateStagingTableRequest,
    ) -> Result<DeltaStagingTableResponse> {
        let url = self.base_url.join(&format!(
            "delta/v1/catalogs/{catalog}/schemas/{schema}/staging-tables"
        ))?;
        let response = self.client.post(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }

    /// Finalize a managed table from its staging reservation —
    /// `POST /delta/v1/catalogs/{catalog}/schemas/{schema}/tables`.
    ///
    /// Called after the client has written `_delta_log/0.json` (carrying the
    /// required features/properties, including `io.unitycatalog.tableId`) to the
    /// staging `location`. The server validates the contract and registers the
    /// table at version 0.
    pub async fn create_table(
        &self,
        catalog: &str,
        schema: &str,
        request: &DeltaCreateTableRequest,
    ) -> Result<DeltaLoadTableResponse> {
        let url = self.base_url.join(&format!(
            "delta/v1/catalogs/{catalog}/schemas/{schema}/tables"
        ))?;
        let response = self.client.post(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }

    /// Load a Delta table's metadata, unbackfilled CCv2 commits, and latest
    /// ratified version — `GET /delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}`.
    ///
    /// For a catalog-managed (MANAGED) table the response carries the `commits`
    /// log tail and `latest_table_version` a reader needs to materialize the
    /// catalog's ratified snapshot.
    pub async fn load_table(
        &self,
        catalog: &str,
        schema: &str,
        table: &str,
    ) -> Result<DeltaLoadTableResponse> {
        let url = self.base_url.join(&format!(
            "delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}"
        ))?;
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }

    /// Update a table — `POST /delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}`.
    ///
    /// This is the catalog-managed commit surface: the `add-commit` action
    /// proposes a staged commit (the server ratifies iff `version == last + 1`,
    /// returning `409` on conflict), and `set-latest-backfilled-version` notifies
    /// the catalog that commits up to a version have been published into
    /// `_delta_log/`. Metadata/property/schema changes ride the same call.
    pub async fn update_table(
        &self,
        catalog: &str,
        schema: &str,
        table: &str,
        request: &DeltaUpdateTableRequest,
    ) -> Result<DeltaLoadTableResponse> {
        let url = self.base_url.join(&format!(
            "delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}"
        ))?;
        let response = self.client.post(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
}
