//! Hybrid proxy server.
//!
//! Composes a local Unity Catalog server with an upstream instance: each REST
//! surface is wired at startup to either the local [`ServerHandler`] or a thin
//! per-surface adapter that forwards requests to the upstream client.
//!
//! ## Authorization
//!
//! Adapters are **not** blind passthroughs. Before forwarding a request they run
//! the *same* [`Policy`] check the local handlers use (via the request types'
//! [`SecuredAction`](unitycatalog_server::api::SecuredAction) impls), and list
//! responses are filtered through the policy on [`Permission::Read`] exactly as
//! the local handlers do. The upstream connection is unauthenticated; all access
//! control is enforced here, which lets the hybrid server act as a test-bed for
//! policy-engine integrations against real upstream data.

use std::sync::Arc;

use async_trait::async_trait;
use axum::Router;
use unitycatalog_client::UnityCatalogClient;
use unitycatalog_client::codegen::catalogs::CatalogClient;
use unitycatalog_client::codegen::schemas::SchemaClient;
use unitycatalog_client::codegen::tables::TableClient;
use unitycatalog_common::models::ResourceName;
use unitycatalog_common::models::catalogs::v1::*;
use unitycatalog_common::models::schemas::v1::*;
use unitycatalog_common::models::tables::v1::*;
use unitycatalog_common::{ResourceExt, ResourceIdent};
use unitycatalog_server::api::catalogs::CatalogHandler;
use unitycatalog_server::api::schemas::SchemaHandler;
use unitycatalog_server::api::tables::TableHandler;
use unitycatalog_server::api::{RequestContext, SecuredAction};
use unitycatalog_server::policy::{Decision, Permission, Policy};
use unitycatalog_server::rest::{
    AuthenticationLayer, Authenticator, create_catalogs_router, create_credentials_router,
    create_external_locations_router, create_functions_router, create_recipients_router,
    create_schemas_router, create_shares_router, create_sharing_router, create_tables_router,
};
use unitycatalog_server::services::ServerHandler;
use unitycatalog_server::{Error, Result};

use crate::config::{RoutingConfig, RoutingMode};

/// Map an upstream client error to the server-side error type.
///
/// The server `Error` is coarser than the client's typed API errors, so several
/// statuses collapse to [`Error::Generic`]. This is acceptable for the
/// proof-of-concept; extend the error type if finer fidelity is needed later.
fn map_upstream_err(err: unitycatalog_client::Error) -> Error {
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

/// Run the policy check that the local handler would run for `request`, then
/// short-circuit with `NotAllowed` if denied.
async fn check<A: SecuredAction, Cx: Send + Sync + 'static>(
    policy: &Arc<dyn Policy<Cx>>,
    request: &A,
    ctx: &Cx,
) -> Result<()> {
    policy.check_required(request, ctx).await
}

/// Filter `items` to those the policy grants `Read`, mirroring
/// `process_resources` but usable with an `Arc<dyn Policy>` (which is not
/// `Sized`, so it cannot satisfy `process_resources`'s generic bound).
async fn filter_readable<R: ResourceExt + Send, Cx: Send + Sync + 'static>(
    policy: &Arc<dyn Policy<Cx>>,
    ctx: &Cx,
    items: &mut Vec<R>,
) -> Result<()> {
    let idents: Vec<ResourceIdent> = items.iter().map(|r| r.into()).collect();
    let mut decisions = policy
        .authorize_many(&idents, &Permission::Read, ctx)
        .await?;
    items.retain(|_| decisions.pop() == Some(Decision::Allow));
    Ok(())
}

// ---------------------------------------------------------------------------
// Catalogs
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct UpstreamCatalogHandler<Cx>
where
    Cx: Send + Sync + 'static,
{
    policy: Arc<dyn Policy<Cx>>,
    client: CatalogClient,
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
        filter_readable(&self.policy, &context, &mut resp.catalogs).await?;
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

#[derive(Clone)]
struct UpstreamSchemaHandler<Cx>
where
    Cx: Send + Sync + 'static,
{
    policy: Arc<dyn Policy<Cx>>,
    client: SchemaClient,
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
        filter_readable(&self.policy, &context, &mut resp.schemas).await?;
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

#[derive(Clone)]
struct UpstreamTableHandler<Cx>
where
    Cx: Send + Sync + 'static,
{
    policy: Arc<dyn Policy<Cx>>,
    client: TableClient,
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
        let mut decisions = self
            .policy
            .authorize_many(&idents, &Permission::Read, &context)
            .await?;
        resp.tables
            .retain(|_| decisions.pop() == Some(Decision::Allow));
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
        filter_readable(&self.policy, &context, &mut resp.tables).await?;
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

/// Run the REST server with selected surfaces proxied to an upstream instance.
///
/// Each resource router is given either the local `handler` or a per-surface
/// upstream adapter, decided by `routing`. Delta Sharing and any surface not
/// marked `upstream` are always served locally.
pub async fn run_server_rest_hybrid<A>(
    host: impl AsRef<str>,
    port: u16,
    handler: ServerHandler<RequestContext>,
    policy: Arc<dyn Policy<RequestContext>>,
    client: UnityCatalogClient,
    routing: &RoutingConfig,
    authenticator: A,
) -> unitycatalog_common::Result<()>
where
    A: Authenticator<unitycatalog_server::policy::Principal> + Clone,
{
    use swagger_ui_dist::{ApiDefinition, OpenApiSource};

    let api_def = ApiDefinition {
        uri_prefix: "/api/2.1/unity-catalog",
        api_definition: OpenApiSource::Inline(include_str!("../../../../openapi/openapi.yaml")),
        title: Some("Unity Catalog API"),
    };
    let sharing_api_def = ApiDefinition {
        uri_prefix: "/api/v1/delta-sharing",
        api_definition: OpenApiSource::Inline(include_str!("../../../../openapi/sharing.yaml")),
        title: Some("Delta Sharing API"),
    };

    let catalogs = match routing.catalogs {
        RoutingMode::Local => create_catalogs_router(handler.clone()),
        RoutingMode::Upstream => create_catalogs_router(UpstreamCatalogHandler {
            policy: policy.clone(),
            client: client.catalogs_client(),
        }),
    };
    let schemas = match routing.schemas {
        RoutingMode::Local => create_schemas_router(handler.clone()),
        RoutingMode::Upstream => create_schemas_router(UpstreamSchemaHandler {
            policy: policy.clone(),
            client: client.schemas_client(),
        }),
    };
    let tables = match routing.tables {
        RoutingMode::Local => create_tables_router(handler.clone()),
        RoutingMode::Upstream => create_tables_router(UpstreamTableHandler {
            policy: policy.clone(),
            client: client.tables_client(),
        }),
    };

    // Remaining surfaces are local-only in v1 (validated upstream of here).
    let api_routes = catalogs
        .merge(schemas)
        .merge(tables)
        .merge(create_credentials_router(handler.clone()))
        .merge(create_external_locations_router(handler.clone()))
        .merge(create_functions_router(handler.clone()))
        .merge(create_recipients_router(handler.clone()))
        .merge(create_shares_router(handler.clone()));

    let router = Router::new()
        .nest("/api/2.1/unity-catalog", api_routes)
        .nest(
            "/api/v1/delta-sharing",
            create_sharing_router(handler.clone()),
        );
    let server = router.layer(AuthenticationLayer::new(authenticator));

    super::run::run(server, host, port, api_def, sharing_api_def).await
}
