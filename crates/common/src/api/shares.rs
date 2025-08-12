use itertools::Itertools;
use std::collections::HashMap;

use super::{RequestContext, SecuredAction};
pub use crate::codegen::{ShareClient, ShareHandler};
use crate::models::ObjectLabel;
use crate::models::shares::v1::*;
use crate::resources::{ResourceIdent, ResourceName, ResourceRef, ResourceStore};
use crate::services::policy::{Permission, Policy, process_resources};
use crate::{Error, Result};

impl SecuredAction for CreateShareRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::share(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Create
    }
}

impl SecuredAction for ListSharesRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::share(ResourceRef::Undefined)
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for GetShareRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::share(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for UpdateShareRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::share(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}

impl SecuredAction for DeleteShareRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::share(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}

#[async_trait::async_trait]
impl<T: ResourceStore + Policy> ShareHandler for T {
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
