#![allow(unused_mut)]
use crate::error::Result;
use cloud_client::CloudClient;
use unitycatalog_common::models::credentials::v1::*;
use url::Url;
/// HTTP client for service operations
#[derive(Clone)]
pub struct CredentialClient {
    pub(crate) client: CloudClient,
    pub(crate) base_url: Url,
}
impl CredentialClient {
    /// Create a new client instance
    pub fn new(client: CloudClient, mut base_url: Url) -> Self {
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        Self { client, base_url }
    }
    pub async fn list_credentials(
        &self,
        request: &ListCredentialsRequest,
    ) -> Result<ListCredentialsResponse> {
        let mut url = self.base_url.join("credentials")?;
        if let Some(ref value) = request.purpose {
            url.query_pairs_mut()
                .append_pair("purpose", &value.to_string());
        }
        if let Some(ref value) = request.max_results {
            url.query_pairs_mut()
                .append_pair("max_results", &value.to_string());
        }
        if let Some(ref value) = request.page_token {
            url.query_pairs_mut()
                .append_pair("page_token", &value.to_string());
        }
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn create_credential(&self, request: &CreateCredentialRequest) -> Result<Credential> {
        let mut url = self.base_url.join("credentials")?;
        let response = self.client.post(url).json(request).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn get_credential(&self, request: &GetCredentialRequest) -> Result<Credential> {
        let formatted_path = format!("credentials/{}", request.name);
        let mut url = self.base_url.join(&formatted_path)?;
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn update_credential(&self, request: &UpdateCredentialRequest) -> Result<Credential> {
        let formatted_path = format!("credentials/{}", request.name);
        let mut url = self.base_url.join(&formatted_path)?;
        let response = self.client.patch(url).json(request).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn delete_credential(&self, request: &DeleteCredentialRequest) -> Result<()> {
        let formatted_path = format!("credentials/{}", request.name);
        let mut url = self.base_url.join(&formatted_path)?;
        let response = self.client.delete(url).send().await?;
        response.error_for_status()?;
        Ok(())
    }
}
