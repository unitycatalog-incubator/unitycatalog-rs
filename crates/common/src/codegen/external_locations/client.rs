#![allow(unused_mut)]
use crate::Result;
use crate::models::external_locations::v1::*;
use cloud_client::CloudClient;
use url::Url;
/// HTTP client for service operations
#[derive(Clone)]
pub struct ExternalLocationClient {
    pub(crate) client: CloudClient,
    pub(crate) base_url: Url,
}
impl ExternalLocationClient {
    /// Create a new client instance
    pub fn new(client: CloudClient, base_url: Url) -> Self {
        Self { client, base_url }
    }
    pub async fn list_external_locations(
        &self,
        request: &ListExternalLocationsRequest,
    ) -> Result<ListExternalLocationsResponse> {
        let mut url = self.base_url.join("/external-locations")?;
        if let Some(ref value) = request.max_results {
            url.query_pairs_mut()
                .append_pair("max_results", &value.to_string());
        }
        if let Some(ref value) = request.page_token {
            url.query_pairs_mut()
                .append_pair("page_token", &value.to_string());
        }
        if let Some(ref value) = request.include_browse {
            url.query_pairs_mut()
                .append_pair("include_browse", &value.to_string());
        }
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn create_external_location(
        &self,
        request: &CreateExternalLocationRequest,
    ) -> Result<ExternalLocationInfo> {
        let mut url = self.base_url.join("/external-locations")?;
        let response = self.client.post(url).json(request).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn get_external_location(
        &self,
        request: &GetExternalLocationRequest,
    ) -> Result<ExternalLocationInfo> {
        let formatted_path = format!("/external-locations/{}", request.name);
        let mut url = self.base_url.join(&formatted_path)?;
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn update_external_location(
        &self,
        request: &UpdateExternalLocationRequest,
    ) -> Result<ExternalLocationInfo> {
        let formatted_path = format!("/external-locations/{}", request.name);
        let mut url = self.base_url.join(&formatted_path)?;
        let response = self.client.patch(url).json(request).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn delete_external_location(
        &self,
        request: &DeleteExternalLocationRequest,
    ) -> Result<()> {
        let formatted_path = format!("/external-locations/{}", request.name);
        let mut url = self.base_url.join(&formatted_path)?;
        if let Some(ref value) = request.force {
            url.query_pairs_mut()
                .append_pair("force", &value.to_string());
        }
        let response = self.client.delete(url).send().await?;
        response.error_for_status()?;
        Ok(())
    }
}
