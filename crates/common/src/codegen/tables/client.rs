use crate::models::tables::v1::*;
use cloud_client::CloudClient;
use url::Url;
/// HTTP client for service operations
#[derive(Clone)]
pub struct TableClient {
    client: CloudClient,
    base_url: Url,
}
impl TableClient {
    /// Create a new client instance
    pub fn new(client: CloudClient, base_url: Url) -> Self {
        Self { client, base_url }
    }
    pub async fn list_table_summaries(
        &self,
        request: &ListTableSummariesRequest,
    ) -> crate::Result<ListTableSummariesResponse> {
        let url = self.base_url.join("/table-summaries")?;
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn list_tables(
        &self,
        request: &ListTablesRequest,
    ) -> crate::Result<ListTablesResponse> {
        let url = self.base_url.join("/tables")?;
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn create_table(&self, request: &CreateTableRequest) -> crate::Result<TableInfo> {
        let url = self.base_url.join("/tables")?;
        let response = self.client.post(url).json(request).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn get_table(&self, request: &GetTableRequest) -> crate::Result<TableInfo> {
        let formatted_path = format!("/tables/{}", request.full_name);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn get_table_exists(
        &self,
        request: &GetTableExistsRequest,
    ) -> crate::Result<GetTableExistsResponse> {
        let formatted_path = format!("/tables/{}/exists", request.full_name);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn delete_table(&self, request: &DeleteTableRequest) -> crate::Result<()> {
        let formatted_path = format!("/tables/{}", request.full_name);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.delete(url).send().await?;
        response.error_for_status()?;
        Ok(())
    }
}
