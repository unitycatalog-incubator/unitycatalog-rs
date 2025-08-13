#![allow(unused_mut)]
use crate::Result;
use crate::models::sharing::v1::*;
use cloud_client::CloudClient;
use url::Url;
/// HTTP client for service operations
#[derive(Clone)]
pub struct SharingClient {
    pub(crate) client: CloudClient,
    pub(crate) base_url: Url,
}
impl SharingClient {
    /// Create a new client instance
    pub fn new(client: CloudClient, base_url: Url) -> Self {
        Self { client, base_url }
    }
    pub async fn list_shares(&self, request: &ListSharesRequest) -> Result<ListSharesResponse> {
        let mut url = self.base_url.join("shares")?;
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
    pub async fn get_share(&self, request: &GetShareRequest) -> Result<Share> {
        let formatted_path = format!("shares/{}", request.name);
        let mut url = self.base_url.join(&formatted_path)?;
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn list_sharing_schemas(
        &self,
        request: &ListSharingSchemasRequest,
    ) -> Result<ListSharingSchemasResponse> {
        let formatted_path = format!("shares/{}/schemas", request.share);
        let mut url = self.base_url.join(&formatted_path)?;
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
    pub async fn list_schema_tables(
        &self,
        request: &ListSchemaTablesRequest,
    ) -> Result<ListSchemaTablesResponse> {
        let formatted_path = format!("shares/{}/schemas/{}/tables", request.share, request.name);
        let mut url = self.base_url.join(&formatted_path)?;
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
    pub async fn list_share_tables(
        &self,
        request: &ListShareTablesRequest,
    ) -> Result<ListShareTablesResponse> {
        let formatted_path = format!("shares/{}/all-tables", request.name);
        let mut url = self.base_url.join(&formatted_path)?;
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
    pub async fn get_table_version(
        &self,
        request: &GetTableVersionRequest,
    ) -> Result<GetTableVersionResponse> {
        let formatted_path = format!(
            "shares/{}/schemas/{}/tables/{}/version",
            request.share, request.schema, request.name
        );
        let mut url = self.base_url.join(&formatted_path)?;
        if let Some(ref value) = request.starting_timestamp {
            url.query_pairs_mut()
                .append_pair("starting_timestamp", &value.to_string());
        }
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn get_table_metadata(
        &self,
        request: &GetTableMetadataRequest,
    ) -> Result<QueryResponse> {
        let formatted_path = format!(
            "shares/{}/schemas/{}/tables/{}/metadata",
            request.share, request.schema, request.name
        );
        let mut url = self.base_url.join(&formatted_path)?;
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn query_table(&self, request: &QueryTableRequest) -> Result<QueryResponse> {
        let formatted_path = format!(
            "shares/{}/schemas/{}/tables/{}/query",
            request.share, request.schema, request.name
        );
        let mut url = self.base_url.join(&formatted_path)?;
        let response = self.client.post(url).json(request).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
}
