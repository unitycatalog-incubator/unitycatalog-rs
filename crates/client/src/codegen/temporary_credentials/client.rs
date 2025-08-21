#![allow(unused_mut)]
use cloud_client::CloudClient;
use url::Url;
use crate::error::Result;
use unitycatalog_common::models::temporary_credentials::v1::*;
/// HTTP client for service operations
#[derive(Clone)]
pub struct TemporaryCredentialClient {
    pub(crate) client: CloudClient,
    pub(crate) base_url: Url,
}
impl TemporaryCredentialClient {
    /// Create a new client instance
    pub fn new(client: CloudClient, mut base_url: Url) -> Self {
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        Self { client, base_url }
    }
    pub async fn generate_temporary_table_credentials(
        &self,
        request: &GenerateTemporaryTableCredentialsRequest,
    ) -> Result<TemporaryCredential> {
        let mut url = self.base_url.join("temporary-table-credentials")?;
        let response = self.client.post(url).json(request).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn generate_temporary_path_credentials(
        &self,
        request: &GenerateTemporaryPathCredentialsRequest,
    ) -> Result<TemporaryCredential> {
        let mut url = self.base_url.join("temporary-path-credentials")?;
        let response = self.client.post(url).json(request).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
}
