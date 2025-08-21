#![allow(unused_mut)]
use cloud_client::CloudClient;
use url::Url;
use crate::error::Result;
use unitycatalog_common::models::catalogs::v1::*;
/// HTTP client for service operations
#[derive(Clone)]
pub struct CatalogClient {
    pub(crate) client: CloudClient,
    pub(crate) base_url: Url,
}
impl CatalogClient {
    /// Create a new client instance
    pub fn new(client: CloudClient, mut base_url: Url) -> Self {
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        Self { client, base_url }
    }
    pub async fn list_catalogs(
        &self,
        request: &ListCatalogsRequest,
    ) -> Result<ListCatalogsResponse> {
        let mut url = self.base_url.join("catalogs")?;
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
    pub async fn create_catalog(
        &self,
        request: &CreateCatalogRequest,
    ) -> Result<CatalogInfo> {
        let mut url = self.base_url.join("catalogs")?;
        let response = self.client.post(url).json(request).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn get_catalog(&self, request: &GetCatalogRequest) -> Result<CatalogInfo> {
        let formatted_path = format!("catalogs/{}", request.name);
        let mut url = self.base_url.join(&formatted_path)?;
        if let Some(ref value) = request.include_browse {
            url.query_pairs_mut().append_pair("include_browse", &value.to_string());
        }
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn update_catalog(
        &self,
        request: &UpdateCatalogRequest,
    ) -> Result<CatalogInfo> {
        let formatted_path = format!("catalogs/{}", request.name);
        let mut url = self.base_url.join(&formatted_path)?;
        let response = self.client.patch(url).json(request).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn delete_catalog(&self, request: &DeleteCatalogRequest) -> Result<()> {
        let formatted_path = format!("catalogs/{}", request.name);
        let mut url = self.base_url.join(&formatted_path)?;
        if let Some(ref value) = request.force {
            url.query_pairs_mut().append_pair("force", &value.to_string());
        }
        let response = self.client.delete(url).send().await?;
        response.error_for_status()?;
        Ok(())
    }
}
