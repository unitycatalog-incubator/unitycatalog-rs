//! Hybrid proxy server wiring.
//!
//! Composes a local Unity Catalog server with an upstream instance: each REST
//! surface is wired at startup to either the local [`ServerHandler`] or a
//! per-surface upstream adapter (from [`unitycatalog_server::handlers::upstream`]),
//! decided by [`RoutingConfig`]. The adapters enforce this server's policy
//! locally before forwarding; see that module for details. Delta Sharing and any
//! surface not marked `upstream` are always served locally.

use std::sync::Arc;

use axum::Router;
use unitycatalog_client::UnityCatalogClient;
use unitycatalog_server::api::RequestContext;
use unitycatalog_server::handlers::upstream::{
    UpstreamCatalogHandler, UpstreamSchemaHandler, UpstreamTableHandler,
};
use unitycatalog_server::policy::Policy;
use unitycatalog_server::rest::{
    AuthenticationLayer, Authenticator, create_catalogs_router, create_credentials_router,
    create_external_locations_router, create_functions_router, create_recipients_router,
    create_schemas_router, create_shares_router, create_sharing_router, create_tables_router,
};
use unitycatalog_server::services::ServerHandler;

use crate::config::{RoutingConfig, RoutingMode};

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
        RoutingMode::Upstream => create_catalogs_router(UpstreamCatalogHandler::new(
            policy.clone(),
            client.catalogs_client(),
        )),
    };
    let schemas = match routing.schemas {
        RoutingMode::Local => create_schemas_router(handler.clone()),
        RoutingMode::Upstream => create_schemas_router(UpstreamSchemaHandler::new(
            policy.clone(),
            client.schemas_client(),
        )),
    };
    let tables = match routing.tables {
        RoutingMode::Local => create_tables_router(handler.clone()),
        RoutingMode::Upstream => create_tables_router(UpstreamTableHandler::new(
            policy.clone(),
            client.tables_client(),
        )),
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
