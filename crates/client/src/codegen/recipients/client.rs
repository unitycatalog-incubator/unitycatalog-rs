#![allow(unused_mut)]
use cloud_client::CloudClient;
use url::Url;
use crate::error::Result;
use unitycatalog_common::models::recipients::v1::*;
/// HTTP client for service operations
#[derive(Clone)]
pub struct RecipientClient {
    pub(crate) client: CloudClient,
    pub(crate) base_url: Url,
}
impl RecipientClient {
    /// Create a new client instance
    pub fn new(client: CloudClient, mut base_url: Url) -> Self {
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        Self { client, base_url }
    }
    pub async fn list_recipients(
        &self,
        request: &ListRecipientsRequest,
    ) -> Result<ListRecipientsResponse> {
        let mut url = self.base_url.join("recipients")?;
        if let Some(ref value) = request.max_results {
            url.query_pairs_mut().append_pair("max_results", &value.to_string());
        }
        if let Some(ref value) = request.page_token {
            url.query_pairs_mut().append_pair("page_token", &value.to_string());
        }
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn create_recipient(
        &self,
        request: &CreateRecipientRequest,
    ) -> Result<RecipientInfo> {
        let mut url = self.base_url.join("recipients")?;
        let response = self.client.post(url).json(request).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn get_recipient(
        &self,
        request: &GetRecipientRequest,
    ) -> Result<RecipientInfo> {
        let formatted_path = format!("recipients/{}", request.name);
        let mut url = self.base_url.join(&formatted_path)?;
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn update_recipient(
        &self,
        request: &UpdateRecipientRequest,
    ) -> Result<RecipientInfo> {
        let formatted_path = format!("recipients/{}", request.name);
        let mut url = self.base_url.join(&formatted_path)?;
        let response = self.client.patch(url).json(request).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn delete_recipient(
        &self,
        request: &DeleteRecipientRequest,
    ) -> Result<()> {
        let formatted_path = format!("recipients/{}", request.name);
        let mut url = self.base_url.join(&formatted_path)?;
        let response = self.client.delete(url).send().await?;
        response.error_for_status()?;
        Ok(())
    }
}
