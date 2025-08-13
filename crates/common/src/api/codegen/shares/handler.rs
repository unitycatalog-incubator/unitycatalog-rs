use crate::Result;
use crate::api::RequestContext;
use crate::models::shares::v1::*;
use async_trait::async_trait;
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
    ) -> Result<ShareInfo>;
    async fn get_share(
        &self,
        request: GetShareRequest,
        context: RequestContext,
    ) -> Result<ShareInfo>;
    async fn update_share(
        &self,
        request: UpdateShareRequest,
        context: RequestContext,
    ) -> Result<ShareInfo>;
    async fn delete_share(
        &self,
        request: DeleteShareRequest,
        context: RequestContext,
    ) -> Result<()>;
}
