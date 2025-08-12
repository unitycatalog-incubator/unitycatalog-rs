use crate::models::recipients::v1::*;
use cloud_client::CloudClient;
use url::Url;
/// HTTP client for service operations
#[derive(Clone)]
pub struct RecipientClient {
    client: CloudClient,
    base_url: Url,
}
impl RecipientClient {
    /// Create a new client instance
    pub fn new(client: CloudClient, base_url: Url) -> Self {
        Self { client, base_url }
    }
    pub async fn list_recipients(
        &self,
        request: &ListRecipientsRequest,
    ) -> crate::Result<ListRecipientsResponse> {
        let url = self.base_url.join("/recipients")?;
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn create_recipient(
        &self,
        request: &CreateRecipientRequest,
    ) -> crate::Result<RecipientInfo> {
        let url = self.base_url.join("/recipients")?;
        let response = self.client.post(url).json(request).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn get_recipient(
        &self,
        request: &GetRecipientRequest,
    ) -> crate::Result<RecipientInfo> {
        let formatted_path = format!("/recipients/{}", request.name);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn update_recipient(
        &self,
        request: &UpdateRecipientRequest,
    ) -> crate::Result<RecipientInfo> {
        let formatted_path = format!("/recipients/{}", request.name);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.patch(url).json(request).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn delete_recipient(&self, request: &DeleteRecipientRequest) -> crate::Result<()> {
        let formatted_path = format!("/recipients/{}", request.name);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.delete(url).send().await?;
        response.error_for_status()?;
        Ok(())
    }
}
