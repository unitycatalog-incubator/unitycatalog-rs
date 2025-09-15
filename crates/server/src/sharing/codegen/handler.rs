use async_trait::async_trait;
use unitycatalog_sharing_client::models::sharing::v1::*;

use crate::Result;
use crate::api::RequestContext;

#[async_trait]
pub trait SharingHandler: Send + Sync + 'static {
    async fn list_shares(
        &self,
        request: ListSharesRequest,
        context: RequestContext,
    ) -> Result<ListSharesResponse>;
    async fn get_share(&self, request: GetShareRequest, context: RequestContext) -> Result<Share>;
    async fn list_sharing_schemas(
        &self,
        request: ListSchemasRequest,
        context: RequestContext,
    ) -> Result<ListSchemasResponse>;
    async fn list_tables(
        &self,
        request: ListTablesRequest,
        context: RequestContext,
    ) -> Result<ListTablesResponse>;
    async fn list_all_tables(
        &self,
        request: ListAllTablesRequest,
        context: RequestContext,
    ) -> Result<ListAllTablesResponse>;
    async fn get_table_version(
        &self,
        request: GetTableVersionRequest,
        context: RequestContext,
    ) -> Result<GetTableVersionResponse>;
    async fn get_table_metadata(
        &self,
        request: GetTableMetadataRequest,
        context: RequestContext,
    ) -> Result<QueryResponse>;
    async fn query_table(
        &self,
        request: QueryTableRequest,
        context: RequestContext,
    ) -> Result<QueryResponse>;
}
