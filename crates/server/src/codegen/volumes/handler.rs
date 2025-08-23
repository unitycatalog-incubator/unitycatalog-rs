use crate::Result;
use crate::api::RequestContext;
use async_trait::async_trait;
use unitycatalog_common::models::volumes::v1::*;
#[async_trait]
pub trait VolumeHandler: Send + Sync + 'static {
    async fn list_volumes(
        &self,
        request: ListVolumesRequest,
        context: RequestContext,
    ) -> Result<ListVolumesResponse>;
    async fn create_volume(
        &self,
        request: CreateVolumeRequest,
        context: RequestContext,
    ) -> Result<VolumeInfo>;
    async fn get_volume(
        &self,
        request: GetVolumeRequest,
        context: RequestContext,
    ) -> Result<VolumeInfo>;
    async fn update_volume(
        &self,
        request: UpdateVolumeRequest,
        context: RequestContext,
    ) -> Result<VolumeInfo>;
    async fn delete_volume(
        &self,
        request: DeleteVolumeRequest,
        context: RequestContext,
    ) -> Result<()>;
}
