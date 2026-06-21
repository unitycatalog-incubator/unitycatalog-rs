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
    AuthenticationLayer, Authenticator, create_catalogs_router, create_commits_router,
    create_credentials_router, create_delta_router, create_entity_tag_assignments_router,
    create_external_locations_router, create_functions_router, create_open_sharing_router,
    create_providers_router, create_recipients_router, create_schemas_router, create_shares_router,
    create_sharing_router, create_staging_tables_router, create_tables_router,
    create_tag_policies_router,
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
    let open_sharing_api_def = ApiDefinition {
        uri_prefix: "/api/v1/open-sharing",
        api_definition: OpenApiSource::Inline(include_str!("../../../../openapi/sharing.yaml")),
        title: Some("Open Sharing API"),
    };
    let delta_api_def = ApiDefinition {
        // Distinct UI prefix so its swagger-ui asset routes don't collide with the
        // main UC API's (see run.rs for the rationale).
        uri_prefix: "/api/2.1/unity-catalog/delta",
        api_definition: OpenApiSource::Inline(include_str!("../../../../openapi/delta.yaml")),
        title: Some("UC Delta API"),
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

    // Delta Sharing resolves a shared table's storage location by looking up the
    // backing Table primitive. When tables are routed upstream, point that
    // resolution at the upstream handler too, so sharing reads work in the
    // side-by-side topology; otherwise the local handler resolves it.
    let sharing_handler = match routing.tables {
        RoutingMode::Local => handler.clone(),
        RoutingMode::Upstream => {
            handler
                .clone()
                .with_table_source(Arc::new(UpstreamTableHandler::new(
                    policy.clone(),
                    client.tables_client(),
                )))
        }
    };

    // Remaining surfaces are local-only in v1 (validated upstream of here).
    let api_routes = catalogs
        .merge(schemas)
        .merge(tables)
        .merge(create_staging_tables_router(handler.clone()))
        .merge(create_credentials_router(handler.clone()))
        .merge(create_external_locations_router(handler.clone()))
        .merge(create_functions_router(handler.clone()))
        .merge(create_recipients_router(handler.clone()))
        .merge(create_providers_router(handler.clone()))
        .merge(create_shares_router(handler.clone()))
        .merge(create_commits_router(handler.clone()))
        .merge(create_delta_router(handler.clone()))
        .merge(create_entity_tag_assignments_router(handler.clone()));

    let router = Router::new()
        .nest("/api/2.1/unity-catalog", api_routes)
        // Tag Policies (governed tag definitions) are local-only and live under /api/2.1.
        .nest("/api/2.1", create_tag_policies_router(handler.clone()))
        .nest(
            "/api/v1/delta-sharing",
            create_sharing_router(sharing_handler.clone()),
        )
        // Open Sharing: superset surface sharing the same tabular handlers and
        // the same (optionally upstream-routed) table-source resolution.
        .nest(
            "/api/v1/open-sharing",
            create_open_sharing_router(sharing_handler),
        );
    let server = router.layer(AuthenticationLayer::new(authenticator));

    super::run::run(
        server,
        host,
        port,
        vec![
            api_def,
            sharing_api_def,
            open_sharing_api_def,
            delta_api_def,
        ],
    )
    .await
}
