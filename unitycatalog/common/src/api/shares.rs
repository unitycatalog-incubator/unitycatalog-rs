use std::collections::HashMap;

use itertools::Itertools;
use unitycatalog_derive::rest_handlers;

use super::{RequestContext, SecuredAction};
use crate::models::ObjectLabel;
use crate::models::shares::v1::*;
use crate::resources::{ResourceIdent, ResourceName, ResourceRef, ResourceStore};
use crate::services::policy::{Permission, Policy, Recipient, process_resources};
use crate::{Error, Result};

rest_handlers!(
    SharesHandler, "shares", [
        CreateShareRequest, Share, Create, ShareInfo;
        ListSharesRequest, Share, Read, ListSharesResponse;
        GetShareRequest, Share, Read, ShareInfo with [
            name: path as String,
            include_shared_data: query as Option<bool>,
        ];
        UpdateShareRequest, Share, Manage, ShareInfo with [
            name: path as String,
        ];
        DeleteShareRequest, Share, Manage with [
            name: path as String
        ];
    ]
);

#[async_trait::async_trait]
pub trait SharesHandler: Send + Sync + 'static {
    /// Create a new share.
    async fn create_share(
        &self,
        request: CreateShareRequest,
        context: RequestContext,
    ) -> Result<ShareInfo>;

    /// Delete a share.
    async fn delete_share(
        &self,
        request: DeleteShareRequest,
        context: RequestContext,
    ) -> Result<()>;

    /// Get a share.
    async fn get_share(
        &self,
        request: GetShareRequest,
        context: RequestContext,
    ) -> Result<ShareInfo>;

    /// List shares.
    async fn list_shares(
        &self,
        request: ListSharesRequest,
        context: RequestContext,
    ) -> Result<ListSharesResponse>;

    /// Update a share.
    async fn update_share(
        &self,
        request: UpdateShareRequest,
        context: RequestContext,
    ) -> Result<ShareInfo>;
}

#[async_trait::async_trait]
impl<T: ResourceStore + Policy> SharesHandler for T {
    async fn create_share(
        &self,
        request: CreateShareRequest,
        context: RequestContext,
    ) -> Result<ShareInfo> {
        self.check_required(&request, context.as_ref()).await?;
        let resource = ShareInfo {
            name: request.name,
            comment: request.comment,
            ..Default::default()
        };
        // TODO:
        // - update the share with the current actor as owner
        // - create updated_* relations
        self.create(resource.into()).await?.0.try_into()
    }

    async fn delete_share(
        &self,
        request: DeleteShareRequest,
        context: RequestContext,
    ) -> Result<()> {
        self.check_required(&request, context.as_ref()).await?;
        self.delete(&request.resource()).await
    }

    async fn list_shares(
        &self,
        request: ListSharesRequest,
        context: RequestContext,
    ) -> Result<ListSharesResponse> {
        self.check_required(&request, context.as_ref()).await?;
        let (mut resources, next_page_token) = self
            .list(
                &ObjectLabel::ShareInfo,
                None,
                request.max_results.map(|v| v as usize),
                request.page_token,
            )
            .await?;
        process_resources(self, context.as_ref(), &Permission::Read, &mut resources).await?;
        Ok(ListSharesResponse {
            shares: resources.into_iter().map(|r| r.try_into()).try_collect()?,
            next_page_token,
        })
    }

    async fn get_share(
        &self,
        request: GetShareRequest,
        context: RequestContext,
    ) -> Result<ShareInfo> {
        self.check_required(&request, context.as_ref()).await?;
        self.get(&request.resource()).await?.0.try_into()
    }

    async fn update_share(
        &self,
        request: UpdateShareRequest,
        context: RequestContext,
    ) -> Result<ShareInfo> {
        self.check_required(&request, context.as_ref()).await?;
        let ident = request.resource();
        let current: ShareInfo = self.get(&ident).await?.0.try_into()?;

        // update the data_objects according to the actions defined in request
        let mut data_objects: HashMap<String, DataObject> = current
            .data_objects
            .into_iter()
            .map(|d| (d.name.clone(), d))
            .collect();
        for update in request.updates.iter() {
            match update.action() {
                Action::Add => {
                    if let Some(obj) = update.data_object.as_ref() {
                        if data_objects.contains_key(&obj.name) {
                            return Err(Error::AlreadyExists);
                        }
                        data_objects.insert(obj.name.clone(), obj.clone());
                    }
                }
                Action::Remove => {
                    if let Some(obj) = update.data_object.as_ref() {
                        data_objects.remove(&obj.name);
                    }
                }
                Action::Update => {
                    if let Some(obj) = update.data_object.as_ref() {
                        if let Some(existing) = data_objects.get_mut(&obj.name) {
                            *existing = obj.clone();
                        } else {
                            return Err(Error::NotFound);
                        }
                    }
                }
                Action::Unspecified => {
                    return Err(Error::InvalidArgument("Unspecified action".to_string()));
                }
            }
        }

        let resource = ShareInfo {
            name: request.new_name.unwrap_or_else(|| request.name.clone()),
            comment: request.comment.or(current.comment),
            owner: request.owner.or(current.owner),
            data_objects: data_objects.into_values().collect(),
            ..Default::default()
        };
        // TODO:
        // - add update_* relations
        self.update(&ident, resource.into()).await?.0.try_into()
    }
}
