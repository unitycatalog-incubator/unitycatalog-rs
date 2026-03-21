use itertools::Itertools;

use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::models::volumes::v1::*;
use unitycatalog_common::models::{ResourceIdent, ResourceName, ResourceRef};

use super::{RequestContext, SecuredAction};
pub use crate::codegen::volumes::VolumeHandler;
use crate::policy::{Permission, Policy, process_resources};
use crate::store::ResourceStore;
use crate::{Error, Result};

#[async_trait::async_trait]
impl<T: ResourceStore + Policy> VolumeHandler for T {
    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn create_volume(
        &self,
        request: CreateVolumeRequest,
        context: RequestContext,
    ) -> Result<Volume> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, context.as_ref()).await?;

        let volume_type =
            VolumeType::try_from(request.volume_type).unwrap_or(VolumeType::Unspecified);

        let storage_location = match volume_type {
            VolumeType::External => {
                // External volumes MUST have an explicit storage location.
                //
                // TODO (production): validate storage_location against registered
                // ExternalLocation records to ensure the caller has access to that path.
                request
                    .storage_location
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| {
                        Error::invalid_argument("storage_location is required for EXTERNAL volumes")
                    })?
            }
            VolumeType::Managed => {
                // Managed volumes derive their storage location from the catalog storage root.
                //
                // TODO: look up the parent catalog to derive:
                //   format!("{}/{}/{}", catalog.storage_root, schema_name, name)
                // For now accept whatever the caller provides (may be empty).
                request.storage_location.unwrap_or_default()
            }
            VolumeType::Unspecified => {
                return Err(Error::invalid_argument(
                    "volume_type must be specified (EXTERNAL or MANAGED)",
                ));
            }
        };

        let full_name = format!(
            "{}.{}.{}",
            request.catalog_name, request.schema_name, request.name
        );
        let resource = Volume {
            full_name,
            name: request.name,
            catalog_name: request.catalog_name,
            schema_name: request.schema_name,
            volume_type: request.volume_type,
            storage_location,
            comment: request.comment,
            ..Default::default()
        };
        Ok(self.create(resource.into()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context))]
    async fn list_volumes(
        &self,
        request: ListVolumesRequest,
        context: RequestContext,
    ) -> Result<ListVolumesResponse> {
        self.check_required(&request, context.as_ref()).await?;
        let (mut resources, next_page_token) = self
            .list(
                &ObjectLabel::Volume,
                Some(&ResourceName::new([
                    &request.catalog_name,
                    &request.schema_name,
                ])),
                request.max_results.map(|v| v as usize),
                request.page_token,
            )
            .await?;
        process_resources(self, context.as_ref(), &Permission::Read, &mut resources).await?;
        Ok(ListVolumesResponse {
            volumes: resources.into_iter().map(|r| r.try_into()).try_collect()?,
            next_page_token,
        })
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn get_volume(
        &self,
        request: GetVolumeRequest,
        context: RequestContext,
    ) -> Result<Volume> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, context.as_ref()).await?;
        Ok(self.get(&request.resource()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn update_volume(
        &self,
        request: UpdateVolumeRequest,
        context: RequestContext,
    ) -> Result<Volume> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, context.as_ref()).await?;
        let ident = request.resource();
        let name = ResourceName::from_naive_str_split(request.name.as_str());
        let [catalog_name, schema_name, volume_name] = name.as_ref() else {
            return Err(Error::invalid_argument(
                "Invalid volume name - expected <catalog_name>.<schema_name>.<volume_name>",
            ));
        };
        let new_name = request.new_name.as_deref().unwrap_or(volume_name);
        let resource = Volume {
            name: new_name.to_owned(),
            comment: request.comment,
            owner: request.owner,
            catalog_name: catalog_name.to_owned(),
            schema_name: schema_name.to_owned(),
            full_name: format!("{}.{}.{}", catalog_name, schema_name, new_name),
            ..Default::default()
        };
        Ok(self.update(&ident, resource.into()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn delete_volume(
        &self,
        request: DeleteVolumeRequest,
        context: RequestContext,
    ) -> Result<()> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, context.as_ref()).await?;
        Ok(self.delete(&request.resource()).await?)
    }
}

impl SecuredAction for CreateVolumeRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::volume(ResourceName::new([
            self.catalog_name.as_str(),
            self.schema_name.as_str(),
            self.name.as_str(),
        ]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Create
    }
}

impl SecuredAction for ListVolumesRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::volume(ResourceRef::Undefined)
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for GetVolumeRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::volume(ResourceName::from_naive_str_split(self.name.as_str()))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for UpdateVolumeRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::volume(ResourceName::from_naive_str_split(self.name.as_str()))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}

impl SecuredAction for DeleteVolumeRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::volume(ResourceName::from_naive_str_split(self.name.as_str()))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}
