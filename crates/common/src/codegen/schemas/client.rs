use crate::models::schemas::v1::*;
use cloud_client::CloudClient;
use url::Url;
/// HTTP client for service operations
#[derive(Clone)]
pub struct SchemaClient {
    client: CloudClient,
    base_url: Url,
}
impl SchemaClient {
    /// Create a new client instance
    pub fn new(client: CloudClient, base_url: Url) -> Self {
        Self { client, base_url }
    }
    pub async fn list_schemas(
        &self,
        request: &ListSchemasRequest,
    ) -> crate::Result<ListSchemasResponse> {
        let url = self.base_url.join("/schemas")?;
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn create_schema(&self, request: &CreateSchemaRequest) -> crate::Result<SchemaInfo> {
        let url = self.base_url.join("/schemas")?;
        let response = self.client.post(url).json(request).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn get_schema(&self, request: &GetSchemaRequest) -> crate::Result<SchemaInfo> {
        let url = self.base_url.join("/schemas/{name}")?;
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn update_schema(&self, request: &UpdateSchemaRequest) -> crate::Result<SchemaInfo> {
        let formatted_path = format!("/schemas/{}", request.full_name);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.patch(url).json(request).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn delete_schema(&self, request: &DeleteSchemaRequest) -> crate::Result<()> {
        let url = self.base_url.join("/schemas/{name}")?;
        let response = self.client.delete(url).send().await?;
        response.error_for_status()?;
        Ok(())
    }
}
