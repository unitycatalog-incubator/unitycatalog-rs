use crate::api::RequestContext;
use crate::models::temporary_credentials::v1::*;
use crate::Result;
use async_trait::async_trait;
#[async_trait]
pub trait TemporaryCredentialHandler: Send + Sync + 'static {
    async fn generate_temporary_table_credentials(
        &self,
        request: GenerateTemporaryTableCredentialsRequest,
        context: RequestContext,
    ) -> Result<TemporaryCredential>;
    async fn generate_temporary_volume_credentials(
        &self,
        request: GenerateTemporaryVolumeCredentialsRequest,
        context: RequestContext,
    ) -> Result<TemporaryCredential>;
}
