use crate::Result;
use crate::api::RequestContext;
use async_trait::async_trait;
use unitycatalog_common::models::shares::v1::*;
#[async_trait]
pub trait ShareHandler: Send + Sync + 'static {
    async fn list_shares(
        &self,
        request: ListSharesRequest,
        context: RequestContext,
    ) -> Result<ListSharesResponse>;
    async fn create_share(
        &self,
        request: CreateShareRequest,
        context: RequestContext,
    ) -> Result<Share>;
    async fn get_share(&self, request: GetShareRequest, context: RequestContext) -> Result<Share>;
    async fn update_share(
        &self,
        request: UpdateShareRequest,
        context: RequestContext,
    ) -> Result<Share>;
    async fn delete_share(
        &self,
        request: DeleteShareRequest,
        context: RequestContext,
    ) -> Result<()>;
    async fn get_permissions(
        &self,
        request: GetPermissionsRequest,
        context: RequestContext,
    ) -> Result<GetPermissionsResponse>;
    async fn update_permissions(
        &self,
        request: UpdatePermissionsRequest,
        context: RequestContext,
    ) -> Result<UpdatePermissionsResponse>;
}
