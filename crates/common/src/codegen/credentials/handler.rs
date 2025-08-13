use crate::Result;
use crate::api::RequestContext;
use crate::models::credentials::v1::*;
use async_trait::async_trait;
#[async_trait]
pub trait CredentialHandler: Send + Sync + 'static {
    async fn list_credentials(
        &self,
        request: ListCredentialsRequest,
        context: RequestContext,
    ) -> Result<ListCredentialsResponse>;
    async fn create_credential(
        &self,
        request: CreateCredentialRequest,
        context: RequestContext,
    ) -> Result<CredentialInfo>;
    async fn get_credential(
        &self,
        request: GetCredentialRequest,
        context: RequestContext,
    ) -> Result<CredentialInfo>;
    async fn update_credential(
        &self,
        request: UpdateCredentialRequest,
        context: RequestContext,
    ) -> Result<CredentialInfo>;
    async fn delete_credential(
        &self,
        request: DeleteCredentialRequest,
        context: RequestContext,
    ) -> Result<()>;
}
