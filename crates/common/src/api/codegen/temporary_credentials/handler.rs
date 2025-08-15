use crate::Result;
use crate::api::RequestContext;
use crate::models::temporary_credentials::v1::*;
use async_trait::async_trait;
#[async_trait]
pub trait TemporaryCredentialHandler: Send + Sync + 'static {
    async fn generate_temporary_table_credentials(
        &self,
        request: GenerateTemporaryTableCredentialsRequest,
        context: RequestContext,
    ) -> Result<TemporaryCredential>;
    async fn generate_temporary_path_credentials(
        &self,
        request: GenerateTemporaryPathCredentialsRequest,
        context: RequestContext,
    ) -> Result<TemporaryCredential>;
}
