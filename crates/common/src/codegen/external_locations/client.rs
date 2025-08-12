use crate::models::external_locations::v1::*;
use cloud_client::CloudClient;
use url::Url;
/// HTTP client for service operations
#[derive(Clone)]
pub struct ExternalLocationClient {
    client: CloudClient,
    base_url: Url,
}
impl ExternalLocationClient {
    /// Create a new client instance
    pub fn new(client: CloudClient, base_url: Url) -> Self {
        Self { client, base_url }
    }
    pub async fn list_external_locations(
        &self,
        request: &ListExternalLocationsRequest,
    ) -> crate::Result<ListExternalLocationsResponse> {
        let url = self.base_url.join("/external-locations")?;
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn create_external_location(
        &self,
        request: &CreateExternalLocationRequest,
    ) -> crate::Result<ExternalLocationInfo> {
        let url = self.base_url.join("/external-locations")?;
        let response = self.client.post(url).json(request).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn get_external_location(
        &self,
        request: &GetExternalLocationRequest,
    ) -> crate::Result<ExternalLocationInfo> {
        let formatted_path = format!("/external-locations/{}", request.name);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn update_external_location(
        &self,
        request: &UpdateExternalLocationRequest,
    ) -> crate::Result<ExternalLocationInfo> {
        let formatted_path = format!("/external-locations/{}", request.name);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.patch(url).json(request).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn delete_external_location(
        &self,
        request: &DeleteExternalLocationRequest,
    ) -> crate::Result<()> {
        let formatted_path = format!("/external-locations/{}", request.name);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.delete(url).send().await?;
        response.error_for_status()?;
        Ok(())
    }
}
