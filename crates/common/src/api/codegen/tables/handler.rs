use crate::Result;
use crate::api::RequestContext;
use crate::models::tables::v1::*;
use async_trait::async_trait;
#[async_trait]
pub trait TableHandler: Send + Sync + 'static {
    async fn list_table_summaries(
        &self,
        request: ListTableSummariesRequest,
        context: RequestContext,
    ) -> Result<ListTableSummariesResponse>;
    async fn list_tables(
        &self,
        request: ListTablesRequest,
        context: RequestContext,
    ) -> Result<ListTablesResponse>;
    async fn create_table(
        &self,
        request: CreateTableRequest,
        context: RequestContext,
    ) -> Result<TableInfo>;
    async fn get_table(
        &self,
        request: GetTableRequest,
        context: RequestContext,
    ) -> Result<TableInfo>;
    async fn get_table_exists(
        &self,
        request: GetTableExistsRequest,
        context: RequestContext,
    ) -> Result<GetTableExistsResponse>;
    async fn delete_table(
        &self,
        request: DeleteTableRequest,
        context: RequestContext,
    ) -> Result<()>;
}
