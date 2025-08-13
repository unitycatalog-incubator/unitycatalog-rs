use crate::Result;
use crate::api::RequestContext;
use crate::models::external_locations::v1::*;
use async_trait::async_trait;
#[async_trait]
pub trait ExternalLocationHandler: Send + Sync + 'static {
    async fn list_external_locations(
        &self,
        request: ListExternalLocationsRequest,
        context: RequestContext,
    ) -> Result<ListExternalLocationsResponse>;
    async fn create_external_location(
        &self,
        request: CreateExternalLocationRequest,
        context: RequestContext,
    ) -> Result<ExternalLocationInfo>;
    async fn get_external_location(
        &self,
        request: GetExternalLocationRequest,
        context: RequestContext,
    ) -> Result<ExternalLocationInfo>;
    async fn update_external_location(
        &self,
        request: UpdateExternalLocationRequest,
        context: RequestContext,
    ) -> Result<ExternalLocationInfo>;
    async fn delete_external_location(
        &self,
        request: DeleteExternalLocationRequest,
        context: RequestContext,
    ) -> Result<()>;
}
