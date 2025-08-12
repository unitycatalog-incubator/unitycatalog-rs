use crate::models::shares::v1::*;
use cloud_client::CloudClient;
use url::Url;
/// HTTP client for service operations
#[derive(Clone)]
pub struct ShareClient {
    pub(crate) client: CloudClient,
    pub(crate) base_url: Url,
}
impl ShareClient {
    /// Create a new client instance
    pub fn new(client: CloudClient, base_url: Url) -> Self {
        Self { client, base_url }
    }
    pub async fn list_shares(
        &self,
        request: &ListSharesRequest,
    ) -> crate::Result<ListSharesResponse> {
        let url = self.base_url.join("/shares")?;
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn create_share(&self, request: &CreateShareRequest) -> crate::Result<ShareInfo> {
        let url = self.base_url.join("/shares")?;
        let response = self.client.post(url).json(request).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn get_share(&self, request: &GetShareRequest) -> crate::Result<ShareInfo> {
        let formatted_path = format!("/shares/{}", request.name);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn update_share(&self, request: &UpdateShareRequest) -> crate::Result<ShareInfo> {
        let formatted_path = format!("/shares/{}", request.name);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.patch(url).json(request).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn delete_share(&self, request: &DeleteShareRequest) -> crate::Result<()> {
        let formatted_path = format!("/shares/{}", request.name);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.delete(url).send().await?;
        response.error_for_status()?;
        Ok(())
    }
}
