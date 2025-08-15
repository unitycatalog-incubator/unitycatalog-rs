#![allow(unused_mut)]
use crate::Result;
use crate::models::tables::v1::*;
use cloud_client::CloudClient;
use url::Url;
/// HTTP client for service operations
#[derive(Clone)]
pub struct TableClient {
    pub(crate) client: CloudClient,
    pub(crate) base_url: Url,
}
impl TableClient {
    /// Create a new client instance
    pub fn new(client: CloudClient, mut base_url: Url) -> Self {
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        Self { client, base_url }
    }
    pub async fn list_table_summaries(
        &self,
        request: &ListTableSummariesRequest,
    ) -> Result<ListTableSummariesResponse> {
        let mut url = self.base_url.join("table-summaries")?;
        url.query_pairs_mut()
            .append_pair("catalog_name", &request.catalog_name.to_string());
        if let Some(ref value) = request.schema_name_pattern {
            url.query_pairs_mut()
                .append_pair("schema_name_pattern", &value.to_string());
        }
        if let Some(ref value) = request.table_name_pattern {
            url.query_pairs_mut()
                .append_pair("table_name_pattern", &value.to_string());
        }
        if let Some(ref value) = request.max_results {
            url.query_pairs_mut()
                .append_pair("max_results", &value.to_string());
        }
        if let Some(ref value) = request.page_token {
            url.query_pairs_mut()
                .append_pair("page_token", &value.to_string());
        }
        if let Some(ref value) = request.include_manifest_capabilities {
            url.query_pairs_mut()
                .append_pair("include_manifest_capabilities", &value.to_string());
        }
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn list_tables(&self, request: &ListTablesRequest) -> Result<ListTablesResponse> {
        let mut url = self.base_url.join("tables")?;
        url.query_pairs_mut()
            .append_pair("schema_name", &request.schema_name.to_string());
        url.query_pairs_mut()
            .append_pair("catalog_name", &request.catalog_name.to_string());
        if let Some(ref value) = request.max_results {
            url.query_pairs_mut()
                .append_pair("max_results", &value.to_string());
        }
        if let Some(ref value) = request.page_token {
            url.query_pairs_mut()
                .append_pair("page_token", &value.to_string());
        }
        if let Some(ref value) = request.include_delta_metadata {
            url.query_pairs_mut()
                .append_pair("include_delta_metadata", &value.to_string());
        }
        if let Some(ref value) = request.omit_columns {
            url.query_pairs_mut()
                .append_pair("omit_columns", &value.to_string());
        }
        if let Some(ref value) = request.omit_properties {
            url.query_pairs_mut()
                .append_pair("omit_properties", &value.to_string());
        }
        if let Some(ref value) = request.omit_username {
            url.query_pairs_mut()
                .append_pair("omit_username", &value.to_string());
        }
        if let Some(ref value) = request.include_browse {
            url.query_pairs_mut()
                .append_pair("include_browse", &value.to_string());
        }
        if let Some(ref value) = request.include_manifest_capabilities {
            url.query_pairs_mut()
                .append_pair("include_manifest_capabilities", &value.to_string());
        }
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn create_table(&self, request: &CreateTableRequest) -> Result<TableInfo> {
        let mut url = self.base_url.join("tables")?;
        let response = self.client.post(url).json(request).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn get_table(&self, request: &GetTableRequest) -> Result<TableInfo> {
        let formatted_path = format!("tables/{}", request.full_name);
        let mut url = self.base_url.join(&formatted_path)?;
        if let Some(ref value) = request.include_delta_metadata {
            url.query_pairs_mut()
                .append_pair("include_delta_metadata", &value.to_string());
        }
        if let Some(ref value) = request.include_browse {
            url.query_pairs_mut()
                .append_pair("include_browse", &value.to_string());
        }
        if let Some(ref value) = request.include_manifest_capabilities {
            url.query_pairs_mut()
                .append_pair("include_manifest_capabilities", &value.to_string());
        }
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn get_table_exists(
        &self,
        request: &GetTableExistsRequest,
    ) -> Result<GetTableExistsResponse> {
        let formatted_path = format!("tables/{}/exists", request.full_name);
        let mut url = self.base_url.join(&formatted_path)?;
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn delete_table(&self, request: &DeleteTableRequest) -> Result<()> {
        let formatted_path = format!("tables/{}", request.full_name);
        let mut url = self.base_url.join(&formatted_path)?;
        let response = self.client.delete(url).send().await?;
        response.error_for_status()?;
        Ok(())
    }
}
