use crate::models::credentials::v1::*;
use cloud_client::CloudClient;
use url::Url;
/// HTTP client for service operations
#[derive(Clone)]
pub struct CredentialClient {
    pub(crate) client: CloudClient,
    pub(crate) base_url: Url,
}
impl CredentialClient {
    /// Create a new client instance
    pub fn new(client: CloudClient, base_url: Url) -> Self {
        Self { client, base_url }
    }
    pub async fn list_credentials(
        &self,
        request: &ListCredentialsRequest,
    ) -> crate::Result<ListCredentialsResponse> {
        let url = self.base_url.join("/credentials")?;
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn create_credential(
        &self,
        request: &CreateCredentialRequest,
    ) -> crate::Result<CredentialInfo> {
        let url = self.base_url.join("/credentials")?;
        let response = self.client.post(url).json(request).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn get_credential(
        &self,
        request: &GetCredentialRequest,
    ) -> crate::Result<CredentialInfo> {
        let formatted_path = format!("/credentials/{}", request.name);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn update_credential(
        &self,
        request: &UpdateCredentialRequest,
    ) -> crate::Result<CredentialInfo> {
        let formatted_path = format!("/credentials/{}", request.name);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.patch(url).json(request).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn delete_credential(&self, request: &DeleteCredentialRequest) -> crate::Result<()> {
        let formatted_path = format!("/credentials/{}", request.name);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.delete(url).send().await?;
        response.error_for_status()?;
        Ok(())
    }
}
