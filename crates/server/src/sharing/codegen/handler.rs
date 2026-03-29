use async_trait::async_trait;
use unitycatalog_sharing_client::models::sharing::v1::*;

use crate::Result;
use crate::api::RequestContext;

#[async_trait]
pub trait SharingHandler<Cx = RequestContext>: Send + Sync + 'static {
    async fn list_shares(
        &self,
        request: ListSharesRequest,
        context: Cx,
    ) -> Result<ListSharesResponse>;
    async fn get_share(&self, request: GetShareRequest, context: Cx) -> Result<Share>;
    async fn list_sharing_schemas(
        &self,
        request: ListSchemasRequest,
        context: Cx,
    ) -> Result<ListSchemasResponse>;
    async fn list_tables(
        &self,
        request: ListTablesRequest,
        context: Cx,
    ) -> Result<ListTablesResponse>;
    async fn list_all_tables(
        &self,
        request: ListAllTablesRequest,
        context: Cx,
    ) -> Result<ListAllTablesResponse>;
    async fn get_table_version(
        &self,
        request: GetTableVersionRequest,
        context: Cx,
    ) -> Result<GetTableVersionResponse>;
    async fn get_table_metadata(
        &self,
        request: GetTableMetadataRequest,
        context: Cx,
    ) -> Result<QueryResponse>;
    async fn query_table(&self, request: QueryTableRequest, context: Cx) -> Result<QueryResponse>;
}
