use crate::Result;
use crate::api::RequestContext;
use async_trait::async_trait;
use unitycatalog_common::models::credentials::v1::*;
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
    ) -> Result<Credential>;
    async fn get_credential(
        &self,
        request: GetCredentialRequest,
        context: RequestContext,
    ) -> Result<Credential>;
    async fn update_credential(
        &self,
        request: UpdateCredentialRequest,
        context: RequestContext,
    ) -> Result<Credential>;
    async fn delete_credential(
        &self,
        request: DeleteCredentialRequest,
        context: RequestContext,
    ) -> Result<()>;
}
