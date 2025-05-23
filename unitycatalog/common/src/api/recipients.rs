use itertools::Itertools;
use unitycatalog_derive::rest_handlers;

use super::{RequestContext, SecuredAction};
use crate::models::ObjectLabel;
use crate::models::recipients::v1::*;
use crate::resources::{ResourceIdent, ResourceName, ResourceRef, ResourceStore};
use crate::services::policy::{Permission, Policy, Recipient, process_resources};
use crate::{Error, Result};

rest_handlers!(
    RecipientsHandler, "recipients", [
        CreateRecipientRequest, Recipient, Create, RecipientInfo;
        ListRecipientsRequest, Recipient, Read, ListRecipientsResponse;
        GetRecipientRequest, Recipient, Read, RecipientInfo with [
            name: path as String,
        ];
        UpdateRecipientRequest, Recipient, Manage, RecipientInfo with [
            name: path as String,
        ];
        DeleteRecipientRequest, Recipient, Manage with [
            name: path as String
        ];
    ]
);

#[async_trait::async_trait]
pub trait RecipientsHandler: Send + Sync + 'static {
    /// List recipients.
    async fn list_recipients(
        &self,
        request: ListRecipientsRequest,
        context: RequestContext,
    ) -> Result<ListRecipientsResponse>;

    /// Create a new recipient.
    async fn create_recipient(
        &self,
        request: CreateRecipientRequest,
        context: RequestContext,
    ) -> Result<RecipientInfo>;

    /// Get a recipient.
    async fn get_recipient(
        &self,
        request: GetRecipientRequest,
        context: RequestContext,
    ) -> Result<RecipientInfo>;

    /// Update a recipient.
    async fn update_recipient(
        &self,
        request: UpdateRecipientRequest,
        context: RequestContext,
    ) -> Result<RecipientInfo>;

    /// Delete a recipient.
    async fn delete_recipient(
        &self,
        request: DeleteRecipientRequest,
        context: RequestContext,
    ) -> Result<()>;
}

#[async_trait::async_trait]
impl<T: ResourceStore + Policy> RecipientsHandler for T {
    async fn create_recipient(
        &self,
        request: CreateRecipientRequest,
        context: RequestContext,
    ) -> Result<RecipientInfo> {
        self.check_required(&request, context.as_ref()).await?;
        let resource = RecipientInfo {
            name: request.name,
            authentication_type: request.authentication_type,
            comment: request.comment,
            properties: request.properties,
            ..Default::default()
        };

        // TODO: create a token placeholder for the recipient with the expiration time etc.
        // this will then later be activated via the activation url

        let info = self.create(resource.into()).await?.0.try_into()?;
        Ok(info)
    }

    async fn delete_recipient(
        &self,
        request: DeleteRecipientRequest,
        context: RequestContext,
    ) -> Result<()> {
        self.check_required(&request, context.as_ref()).await?;
        self.delete(&request.resource()).await
    }

    async fn get_recipient(
        &self,
        request: GetRecipientRequest,
        context: RequestContext,
    ) -> Result<RecipientInfo> {
        self.check_required(&request, context.recipient()).await?;
        self.get(&request.resource()).await?.0.try_into()
    }

    async fn list_recipients(
        &self,
        request: ListRecipientsRequest,
        context: RequestContext,
    ) -> Result<ListRecipientsResponse> {
        self.check_required(&request, context.as_ref()).await?;
        let (mut resources, next_page_token) = self
            .list(
                &ObjectLabel::RecipientInfo,
                None,
                request.max_results.map(|v| v as usize),
                request.page_token,
            )
            .await?;
        process_resources(self, context.as_ref(), &Permission::Read, &mut resources).await?;
        Ok(ListRecipientsResponse {
            recipients: resources.into_iter().map(|r| r.try_into()).try_collect()?,
            next_page_token,
        })
    }

    async fn update_recipient(
        &self,
        _request: UpdateRecipientRequest,
        _context: RequestContext,
    ) -> Result<RecipientInfo> {
        // TODO: once we have token handling, we can update token expiration etc...
        todo!("update_recipient")
    }
}
