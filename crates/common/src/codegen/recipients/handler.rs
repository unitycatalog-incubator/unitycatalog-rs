use crate::api::RequestContext;
use crate::models::recipients::v1::*;
use crate::Result;
use async_trait::async_trait;
#[async_trait]
pub trait RecipientHandler: Send + Sync + 'static {
    async fn list_recipients(
        &self,
        request: ListRecipientsRequest,
        context: RequestContext,
    ) -> Result<ListRecipientsResponse>;
    async fn create_recipient(
        &self,
        request: CreateRecipientRequest,
        context: RequestContext,
    ) -> Result<RecipientInfo>;
    async fn get_recipient(
        &self,
        request: GetRecipientRequest,
        context: RequestContext,
    ) -> Result<RecipientInfo>;
    async fn update_recipient(
        &self,
        request: UpdateRecipientRequest,
        context: RequestContext,
    ) -> Result<RecipientInfo>;
    async fn delete_recipient(
        &self,
        request: DeleteRecipientRequest,
        context: RequestContext,
    ) -> Result<()>;
}
