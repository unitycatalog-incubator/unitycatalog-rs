use crate::models::temporary_credentials::v1::*;
use cloud_client::CloudClient;
use url::Url;
/// HTTP client for service operations
#[derive(Clone)]
pub struct TemporaryCredentialClient {
    pub(crate) client: CloudClient,
    pub(crate) base_url: Url,
}
impl TemporaryCredentialClient {
    /// Create a new client instance
    pub fn new(client: CloudClient, base_url: Url) -> Self {
        Self { client, base_url }
    }
    pub async fn generate_temporary_table_credentials(
        &self,
        request: &GenerateTemporaryTableCredentialsRequest,
    ) -> crate::Result<TemporaryCredential> {
        let url = self.base_url.join("/temporary-table-credentials")?;
        let response = self.client.post(url).json(request).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn generate_temporary_volume_credentials(
        &self,
        request: &GenerateTemporaryVolumeCredentialsRequest,
    ) -> crate::Result<TemporaryCredential> {
        let url = self.base_url.join("/temporary-volume-credentials")?;
        let response = self.client.post(url).json(request).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
}
