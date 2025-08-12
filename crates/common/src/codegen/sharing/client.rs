use crate::models::sharing::v1::*;
use cloud_client::CloudClient;
use url::Url;
/// HTTP client for service operations
#[derive(Clone)]
pub struct SharingClient {
    client: CloudClient,
    base_url: Url,
}
impl SharingClient {
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
    pub async fn get_share(&self, request: &GetShareRequest) -> crate::Result<Share> {
        let formatted_path = format!("/shares/{}", request.name);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn list_sharing_schemas(
        &self,
        request: &ListSharingSchemasRequest,
    ) -> crate::Result<ListSharingSchemasResponse> {
        let formatted_path = format!("/shares/{}/schemas", request.share);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn list_schema_tables(
        &self,
        request: &ListSchemaTablesRequest,
    ) -> crate::Result<ListSchemaTablesResponse> {
        let formatted_path = format!("/shares/{}/schemas/{}/tables", request.share, request.name);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn list_share_tables(
        &self,
        request: &ListShareTablesRequest,
    ) -> crate::Result<ListShareTablesResponse> {
        let formatted_path = format!("/shares/{}/all-tables", request.name);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn get_table_version(
        &self,
        request: &GetTableVersionRequest,
    ) -> crate::Result<GetTableVersionResponse> {
        let formatted_path = format!(
            "/shares/{}/schemas/{}/tables/{}/version",
            request.share, request.schema, request.name
        );
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn get_table_metadata(
        &self,
        request: &GetTableMetadataRequest,
    ) -> crate::Result<QueryResponse> {
        let formatted_path = format!(
            "/shares/{}/schemas/{}/tables/{}/metadata",
            request.share, request.schema, request.name
        );
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn query_table(&self, request: &QueryTableRequest) -> crate::Result<QueryResponse> {
        let formatted_path = format!(
            "/shares/{}/schemas/{}/tables/{}/query",
            request.share, request.schema, request.name
        );
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.post(url).json(request).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
}
