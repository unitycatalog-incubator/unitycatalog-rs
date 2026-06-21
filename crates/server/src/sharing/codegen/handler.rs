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

/// Open Sharing volume APIs: discovery and temporary-credential vending for
/// shared storage-backed volumes.
#[async_trait]
pub trait SharingVolumeHandler<Cx = RequestContext>: Send + Sync + 'static {
    async fn list_volumes(
        &self,
        request: ListVolumesRequest,
        context: Cx,
    ) -> Result<ListVolumesResponse>;
    async fn list_all_volumes(
        &self,
        request: ListAllVolumesRequest,
        context: Cx,
    ) -> Result<ListAllVolumesResponse>;
    async fn get_volume(&self, request: GetVolumeRequest, context: Cx) -> Result<SharingVolume>;
    async fn generate_temporary_volume_credentials(
        &self,
        request: GenerateTemporaryVolumeCredentialsRequest,
        context: Cx,
    ) -> Result<SharingTemporaryCredentials>;
}

/// Open Sharing agent-skill APIs: discovery and temporary-credential vending for
/// shared storage-backed agent skills.
#[async_trait]
pub trait SharingSkillHandler<Cx = RequestContext>: Send + Sync + 'static {
    async fn list_skills(
        &self,
        request: ListSkillsRequest,
        context: Cx,
    ) -> Result<ListSkillsResponse>;
    async fn list_all_skills(
        &self,
        request: ListAllSkillsRequest,
        context: Cx,
    ) -> Result<ListAllSkillsResponse>;
    async fn get_skill(&self, request: GetSkillRequest, context: Cx) -> Result<SharingSkill>;
    async fn generate_temporary_skill_credentials(
        &self,
        request: GenerateTemporarySkillCredentialsRequest,
        context: Cx,
    ) -> Result<SharingTemporaryCredentials>;
}
