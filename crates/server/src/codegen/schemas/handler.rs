use crate::Result;
use crate::api::RequestContext;
use async_trait::async_trait;
use unitycatalog_common::models::schemas::v1::*;
#[async_trait]
pub trait SchemaHandler: Send + Sync + 'static {
    async fn list_schemas(
        &self,
        request: ListSchemasRequest,
        context: RequestContext,
    ) -> Result<ListSchemasResponse>;
    async fn create_schema(
        &self,
        request: CreateSchemaRequest,
        context: RequestContext,
    ) -> Result<Schema>;
    async fn get_schema(
        &self,
        request: GetSchemaRequest,
        context: RequestContext,
    ) -> Result<Schema>;
    async fn update_schema(
        &self,
        request: UpdateSchemaRequest,
        context: RequestContext,
    ) -> Result<Schema>;
    async fn delete_schema(
        &self,
        request: DeleteSchemaRequest,
        context: RequestContext,
    ) -> Result<()>;
}
