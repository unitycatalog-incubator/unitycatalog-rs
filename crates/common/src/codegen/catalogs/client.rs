use crate::models::catalogs::v1::*;
use cloud_client::CloudClient;
use url::Url;
/// HTTP client for service operations
#[derive(Clone)]
pub struct CatalogClient {
    client: CloudClient,
    base_url: Url,
}
impl CatalogClient {
    /// Create a new client instance
    pub fn new(client: CloudClient, base_url: Url) -> Self {
        Self { client, base_url }
    }
    pub async fn list_catalogs(
        &self,
        request: &ListCatalogsRequest,
    ) -> crate::Result<ListCatalogsResponse> {
        let url = self.base_url.join("/catalogs")?;
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn create_catalog(
        &self,
        request: &CreateCatalogRequest,
    ) -> crate::Result<CatalogInfo> {
        let url = self.base_url.join("/catalogs")?;
        let response = self.client.post(url).json(request).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn get_catalog(&self, request: &GetCatalogRequest) -> crate::Result<CatalogInfo> {
        let formatted_path = format!("/catalogs/{}", request.name);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn update_catalog(
        &self,
        request: &UpdateCatalogRequest,
    ) -> crate::Result<CatalogInfo> {
        let formatted_path = format!("/catalogs/{}", request.name);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.patch(url).json(request).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn delete_catalog(&self, request: &DeleteCatalogRequest) -> crate::Result<()> {
        let formatted_path = format!("/catalogs/{}", request.name);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.delete(url).send().await?;
        response.error_for_status()?;
        Ok(())
    }
}
