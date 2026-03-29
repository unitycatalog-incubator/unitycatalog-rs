use itertools::Itertools;

use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::models::schemas::v1::*;
use unitycatalog_common::models::{ResourceIdent, ResourceName, ResourceRef};

use super::{RequestContext, SecuredAction};
pub use crate::codegen::schemas::SchemaHandler;
use crate::policy::{Permission, Policy, process_resources};
use crate::store::ResourceStore;
use crate::{Error, Result};

#[async_trait::async_trait]
impl<T: ResourceStore + Policy<RequestContext>> SchemaHandler<RequestContext> for T {
    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn create_schema(
        &self,
        request: CreateSchemaRequest,
        context: RequestContext,
    ) -> Result<Schema> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        let resource = Schema {
            full_name: format!("{}.{}", request.catalog_name, request.name),
            name: request.name,
            catalog_name: request.catalog_name,
            comment: request.comment,
            properties: request.properties,
            ..Default::default()
        };
        // TODO:
        // - update the schema with the current actor as owner
        // - create updated_* relations
        Ok(self.create(resource.into()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn delete_schema(
        &self,
        request: DeleteSchemaRequest,
        context: RequestContext,
    ) -> Result<()> {
        tracing::Span::current().record("resource_name", &request.full_name);
        self.check_required(&request, &context).await?;
        Ok(self.delete(&request.resource()).await?)
    }

    #[tracing::instrument(skip(self, context))]
    async fn list_schemas(
        &self,
        request: ListSchemasRequest,
        context: RequestContext,
    ) -> Result<ListSchemasResponse> {
        self.check_required(&request, &context).await?;
        let (mut resources, next_page_token) = self
            .list(
                &ObjectLabel::Schema,
                Some(&ResourceName::new([&request.catalog_name])),
                request.max_results.map(|v| v as usize),
                request.page_token,
            )
            .await?;
        process_resources(self, &context, &Permission::Read, &mut resources).await?;
        Ok(ListSchemasResponse {
            schemas: resources.into_iter().map(|r| r.try_into()).try_collect()?,
            next_page_token,
        })
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn get_schema(
        &self,
        request: GetSchemaRequest,
        context: RequestContext,
    ) -> Result<Schema> {
        tracing::Span::current().record("resource_name", &request.full_name);
        self.check_required(&request, &context).await?;
        Ok(self.get(&request.resource()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn update_schema(
        &self,
        request: UpdateSchemaRequest,
        context: RequestContext,
    ) -> Result<Schema> {
        tracing::Span::current().record("resource_name", &request.full_name);
        self.check_required(&request, &context).await?;
        let ident = request.resource();
        let name = ResourceName::from_naive_str_split(request.full_name);
        let [catalog_name, schema_name] = name.as_ref() else {
            return Err(Error::invalid_argument(
                "Invalid schema name - expected <catalog_name>.<schema_name>",
            ));
        };
        let new_name = request.new_name.unwrap_or(schema_name.to_owned());
        let resource = Schema {
            name: new_name.clone(),
            comment: request.comment,
            properties: request.properties,
            catalog_name: catalog_name.to_owned(),
            full_name: format!("{}.{}", catalog_name, new_name),
            ..Default::default()
        };
        // TODO:
        // - add update_* relations
        // - update owner if necessary
        Ok(self.update(&ident, resource.into()).await?.0.try_into()?)
    }
}

impl SecuredAction for CreateSchemaRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::schema(ResourceName::new([
            self.catalog_name.as_str(),
            self.name.as_str(),
        ]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Create
    }
}

impl SecuredAction for ListSchemasRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::schema(ResourceRef::Undefined)
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for GetSchemaRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::schema(ResourceName::from_naive_str_split(self.full_name.as_str()))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for UpdateSchemaRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::schema(ResourceName::from_naive_str_split(self.full_name.as_str()))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}

impl SecuredAction for DeleteSchemaRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::schema(ResourceName::from_naive_str_split(self.full_name.as_str()))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}
