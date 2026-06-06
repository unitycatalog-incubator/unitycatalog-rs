//! Upstream proxy handlers.
//!
//! Per-resource handler implementations that forward requests to an upstream
//! Unity Catalog instance via [`unitycatalog_client`], while enforcing this
//! server's [`Policy`] locally: every request is policy-checked before it is
//! forwarded, and `list_*` responses are filtered through the policy on
//! [`Permission::Read`] — so a proxied surface never returns resources the
//! caller is not allowed to see.
//!
//! These are **leaves** (they terminate at an external backend), generic over
//! the request context `Cx`, and do not reference
//! [`RequestContext`](crate::api::RequestContext).

use std::sync::Arc;

use async_trait::async_trait;
use unitycatalog_client::codegen::catalogs::CatalogServiceClient;
use unitycatalog_client::codegen::schemas::SchemaServiceClient;
use unitycatalog_client::codegen::tables::TableServiceClient;
use unitycatalog_common::ResourceIdent;
use unitycatalog_common::models::ResourceName;
use unitycatalog_common::models::catalogs::v1::*;
use unitycatalog_common::models::schemas::v1::*;
use unitycatalog_common::models::tables::v1::*;

use crate::api::SecuredAction;
use crate::api::catalogs::CatalogHandler;
use crate::api::schemas::SchemaHandler;
use crate::api::tables::TableHandler;
use crate::policy::{Decision, Permission, Policy, filter_authorized};
use crate::{Error, Result};

/// Map an upstream client error to the server-side error type.
///
/// The server `Error` is coarser than the client's typed API errors, so several
/// statuses collapse to [`Error::Generic`]. Extend if finer fidelity is needed.
pub fn map_upstream_err(err: unitycatalog_client::Error) -> Error {
    use unitycatalog_client::UcApiError;
    match err {
        unitycatalog_client::Error::Api(UcApiError::NotFound { .. }) => {
            unitycatalog_common::Error::NotFound.into()
        }
        unitycatalog_client::Error::Api(UcApiError::InvalidParameter { message }) => {
            unitycatalog_common::Error::InvalidArgument(message).into()
        }
        other => Error::Generic(other.to_string()),
    }
}

/// Run the policy check the local handler would run for `request`.
async fn check<A: SecuredAction, Cx: Send + Sync + 'static>(
    policy: &Arc<dyn Policy<Cx>>,
    request: &A,
    ctx: &Cx,
) -> Result<()> {
    policy.check_required(request, ctx).await
}

// ---------------------------------------------------------------------------
// Catalogs
// ---------------------------------------------------------------------------

/// [`CatalogHandler`] that forwards to an upstream Unity Catalog instance.
#[derive(Clone)]
pub struct UpstreamCatalogHandler<Cx>
where
    Cx: Send + Sync + 'static,
{
    policy: Arc<dyn Policy<Cx>>,
    client: CatalogServiceClient,
}

impl<Cx: Send + Sync + 'static> UpstreamCatalogHandler<Cx> {
    pub fn new(policy: Arc<dyn Policy<Cx>>, client: CatalogServiceClient) -> Self {
        Self { policy, client }
    }
}

#[async_trait]
impl<Cx: Send + Sync + 'static> CatalogHandler<Cx> for UpstreamCatalogHandler<Cx> {
    async fn list_catalogs(
        &self,
        request: ListCatalogsRequest,
        context: Cx,
    ) -> Result<ListCatalogsResponse> {
        check(&self.policy, &request, &context).await?;
        let mut resp = self
            .client
            .list_catalogs(&request)
            .await
            .map_err(map_upstream_err)?;
        filter_authorized(
            &*self.policy,
            &context,
            &Permission::Read,
            &mut resp.catalogs,
        )
        .await?;
        Ok(resp)
    }

    async fn create_catalog(&self, request: CreateCatalogRequest, context: Cx) -> Result<Catalog> {
        check(&self.policy, &request, &context).await?;
        self.client
            .create_catalog(&request)
            .await
            .map_err(map_upstream_err)
    }

    async fn get_catalog(&self, request: GetCatalogRequest, context: Cx) -> Result<Catalog> {
        check(&self.policy, &request, &context).await?;
        self.client
            .get_catalog(&request)
            .await
            .map_err(map_upstream_err)
    }

    async fn update_catalog(&self, request: UpdateCatalogRequest, context: Cx) -> Result<Catalog> {
        check(&self.policy, &request, &context).await?;
        self.client
            .update_catalog(&request)
            .await
            .map_err(map_upstream_err)
    }

    async fn delete_catalog(&self, request: DeleteCatalogRequest, context: Cx) -> Result<()> {
        check(&self.policy, &request, &context).await?;
        self.client
            .delete_catalog(&request)
            .await
            .map_err(map_upstream_err)
    }
}

// ---------------------------------------------------------------------------
// Schemas
// ---------------------------------------------------------------------------

/// [`SchemaHandler`] that forwards to an upstream Unity Catalog instance.
#[derive(Clone)]
pub struct UpstreamSchemaHandler<Cx>
where
    Cx: Send + Sync + 'static,
{
    policy: Arc<dyn Policy<Cx>>,
    client: SchemaServiceClient,
}

impl<Cx: Send + Sync + 'static> UpstreamSchemaHandler<Cx> {
    pub fn new(policy: Arc<dyn Policy<Cx>>, client: SchemaServiceClient) -> Self {
        Self { policy, client }
    }
}

#[async_trait]
impl<Cx: Send + Sync + 'static> SchemaHandler<Cx> for UpstreamSchemaHandler<Cx> {
    async fn list_schemas(
        &self,
        request: ListSchemasRequest,
        context: Cx,
    ) -> Result<ListSchemasResponse> {
        check(&self.policy, &request, &context).await?;
        let mut resp = self
            .client
            .list_schemas(&request)
            .await
            .map_err(map_upstream_err)?;
        filter_authorized(
            &*self.policy,
            &context,
            &Permission::Read,
            &mut resp.schemas,
        )
        .await?;
        Ok(resp)
    }

    async fn create_schema(&self, request: CreateSchemaRequest, context: Cx) -> Result<Schema> {
        check(&self.policy, &request, &context).await?;
        self.client
            .create_schema(&request)
            .await
            .map_err(map_upstream_err)
    }

    async fn get_schema(&self, request: GetSchemaRequest, context: Cx) -> Result<Schema> {
        check(&self.policy, &request, &context).await?;
        self.client
            .get_schema(&request)
            .await
            .map_err(map_upstream_err)
    }

    async fn update_schema(&self, request: UpdateSchemaRequest, context: Cx) -> Result<Schema> {
        check(&self.policy, &request, &context).await?;
        self.client
            .update_schema(&request)
            .await
            .map_err(map_upstream_err)
    }

    async fn delete_schema(&self, request: DeleteSchemaRequest, context: Cx) -> Result<()> {
        check(&self.policy, &request, &context).await?;
        self.client
            .delete_schema(&request)
            .await
            .map_err(map_upstream_err)
    }
}

// ---------------------------------------------------------------------------
// Tables
// ---------------------------------------------------------------------------

/// [`TableHandler`] that forwards to an upstream Unity Catalog instance.
#[derive(Clone)]
pub struct UpstreamTableHandler<Cx>
where
    Cx: Send + Sync + 'static,
{
    policy: Arc<dyn Policy<Cx>>,
    client: TableServiceClient,
}

impl<Cx: Send + Sync + 'static> UpstreamTableHandler<Cx> {
    pub fn new(policy: Arc<dyn Policy<Cx>>, client: TableServiceClient) -> Self {
        Self { policy, client }
    }
}

#[async_trait]
impl<Cx: Send + Sync + 'static> TableHandler<Cx> for UpstreamTableHandler<Cx> {
    async fn list_table_summaries(
        &self,
        request: ListTableSummariesRequest,
        context: Cx,
    ) -> Result<ListTableSummariesResponse> {
        check(&self.policy, &request, &context).await?;
        let mut resp = self
            .client
            .list_table_summaries(&request)
            .await
            .map_err(map_upstream_err)?;
        // `TableSummary` does not implement `ResourceExt`, so filter by
        // reconstructing the table ident from its full name (mirrors the
        // `GetTableRequest` SecuredAction).
        let idents: Vec<ResourceIdent> = resp
            .tables
            .iter()
            .map(|t| ResourceIdent::table(ResourceName::from_naive_str_split(&t.full_name)))
            .collect();
        let decisions = self
            .policy
            .authorize_many(&idents, &Permission::Read, &context)
            .await?;
        // `decisions[i]` corresponds to `resp.tables[i]`; pair in forward order.
        let mut allow = decisions.into_iter().map(|d| d == Decision::Allow);
        resp.tables.retain(|_| allow.next().unwrap_or(false));
        Ok(resp)
    }

    async fn list_tables(
        &self,
        request: ListTablesRequest,
        context: Cx,
    ) -> Result<ListTablesResponse> {
        check(&self.policy, &request, &context).await?;
        let mut resp = self
            .client
            .list_tables(&request)
            .await
            .map_err(map_upstream_err)?;
        filter_authorized(&*self.policy, &context, &Permission::Read, &mut resp.tables).await?;
        Ok(resp)
    }

    async fn create_table(&self, request: CreateTableRequest, context: Cx) -> Result<Table> {
        check(&self.policy, &request, &context).await?;
        self.client
            .create_table(&request)
            .await
            .map_err(map_upstream_err)
    }

    async fn get_table(&self, request: GetTableRequest, context: Cx) -> Result<Table> {
        check(&self.policy, &request, &context).await?;
        self.client
            .get_table(&request)
            .await
            .map_err(map_upstream_err)
    }

    async fn get_table_exists(
        &self,
        request: GetTableExistsRequest,
        context: Cx,
    ) -> Result<GetTableExistsResponse> {
        check(&self.policy, &request, &context).await?;
        self.client
            .get_table_exists(&request)
            .await
            .map_err(map_upstream_err)
    }

    async fn delete_table(&self, request: DeleteTableRequest, context: Cx) -> Result<()> {
        check(&self.policy, &request, &context).await?;
        self.client
            .delete_table(&request)
            .await
            .map_err(map_upstream_err)
    }
}
