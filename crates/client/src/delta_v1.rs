//! Hand-written client for the UC Delta REST API (`/delta/v1/...`).
//!
//! The Delta API is a standalone REST protocol (not a generated resource API),
//! so — like [`crate::temporary_credentials`] — it is hand-maintained. The wire
//! DTOs are shared with the server via
//! [`unitycatalog_common::models::delta::v1`]. This client currently covers the
//! read path (`loadTable`); the write/commit surface is a follow-up.

use olai_http::CloudClient;
use unitycatalog_common::models::delta::v1::DeltaLoadTableResponse;
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
}
