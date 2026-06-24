use axum::Router;
use swagger_ui_dist::{ApiDefinition, OpenApiSource};
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::LatencyUnit;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;
use unitycatalog_common::{Error, Result};
use unitycatalog_server::api::agent_skills::AgentSkillHandler;
use unitycatalog_server::api::agents::AgentHandler;
use unitycatalog_server::api::catalogs::CatalogHandler;
use unitycatalog_server::api::commits::DeltaCommitHandler;
use unitycatalog_server::api::credentials::CredentialHandler;
use unitycatalog_server::api::delta::DeltaApiHandler;
use unitycatalog_server::api::entity_tag_assignments::EntityTagAssignmentHandler;
use unitycatalog_server::api::external_locations::ExternalLocationHandler;
use unitycatalog_server::api::functions::FunctionHandler;
use unitycatalog_server::api::providers::ProviderHandler;
use unitycatalog_server::api::recipients::RecipientHandler;
use unitycatalog_server::api::schemas::SchemaHandler;
use unitycatalog_server::api::shares::ShareHandler;
use unitycatalog_server::api::sharing::{SharingHandler, SharingQueryHandler};
use unitycatalog_server::api::staging_tables::StagingTableHandler;
use unitycatalog_server::api::tables::TableHandler;
use unitycatalog_server::api::tag_policies::TagPolicyHandler;
use unitycatalog_server::api::temporary_credentials::TemporaryCredentialHandler;
use unitycatalog_server::api::volumes::VolumeHandler;
use unitycatalog_server::rest::{
    AuthenticationLayer, Authenticator, create_agent_skills_router, create_agents_router,
    create_catalogs_router, create_commits_router, create_credentials_router, create_delta_router,
    create_entity_tag_assignments_router, create_external_locations_router,
    create_functions_router, create_open_sharing_router, create_providers_router,
    create_recipients_router, create_schemas_router, create_shares_router, create_sharing_router,
    create_staging_tables_router, create_tables_router, create_tag_policies_router,
    create_temporary_credentials_router, create_volumes_router,
};
use unitycatalog_server::sharing::{SharingSkillHandler, SharingVolumeHandler};

pub async fn run_server_rest<T, A, Cx>(
    host: impl AsRef<str>,
    port: u16,
    handler: T,
    authenticator: A,
) -> Result<()>
where
    T: CatalogHandler<Cx>
        + CredentialHandler<Cx>
        + FunctionHandler<Cx>
        + SharingHandler<Cx>
        + SharingQueryHandler<Cx>
        + SharingVolumeHandler<Cx>
        + SharingSkillHandler<Cx>
        + ShareHandler<Cx>
        + SchemaHandler<Cx>
        + StagingTableHandler<Cx>
        + TableHandler<Cx>
        + VolumeHandler<Cx>
        + AgentSkillHandler<Cx>
        + AgentHandler<Cx>
        + ExternalLocationHandler<Cx>
        + RecipientHandler<Cx>
        + ProviderHandler<Cx>
        + DeltaCommitHandler<Cx>
        + DeltaApiHandler<Cx>
        + TagPolicyHandler<Cx>
        + EntityTagAssignmentHandler<Cx>
        + TemporaryCredentialHandler<Cx>
        + Clone,
    A: Authenticator<unitycatalog_server::policy::Principal> + Clone,
    Cx: axum::extract::FromRequestParts<T> + Send + 'static,
{
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
    // Open Sharing is a superset of Delta Sharing served at its own prefix; the
    // tabular surface is wire-compatible, so it currently reuses the same spec.
    let open_sharing_api_def = ApiDefinition {
        uri_prefix: "/api/v1/open-sharing",
        api_definition: OpenApiSource::Inline(include_str!("../../../../openapi/sharing.yaml")),
        title: Some("Open Sharing API"),
    };
    // The Delta REST API routes live at `/delta/v1/...` under the UC base path, but its
    // Swagger UI + spec are hosted under a distinct prefix so the swagger-ui asset routes
    // (`swagger-ui.css`, etc.) don't collide with the main UC API's, which is mounted at
    // `/api/2.1/unity-catalog`. (The spec's own `servers` block still advertises the real
    // base path to clients.)
    let delta_api_def = ApiDefinition {
        uri_prefix: "/api/2.1/unity-catalog/delta",
        api_definition: OpenApiSource::Inline(include_str!("../../../../openapi/delta.yaml")),
        title: Some("UC Delta API"),
    };

    let api_routes = create_catalogs_router(handler.clone())
        .merge(create_schemas_router(handler.clone()))
        .merge(create_staging_tables_router(handler.clone()))
        .merge(create_tables_router(handler.clone()))
        .merge(create_volumes_router(handler.clone()))
        .merge(create_agent_skills_router(handler.clone()))
        .merge(create_agents_router(handler.clone()))
        .merge(create_credentials_router(handler.clone()))
        .merge(create_external_locations_router(handler.clone()))
        .merge(create_temporary_credentials_router(handler.clone()))
        .merge(create_functions_router(handler.clone()))
        .merge(create_recipients_router(handler.clone()))
        .merge(create_providers_router(handler.clone()))
        .merge(create_shares_router(handler.clone()))
        .merge(create_commits_router(handler.clone()))
        .merge(create_delta_router(handler.clone()))
        .merge(create_entity_tag_assignments_router(handler.clone()));

    let router = Router::new()
        .nest("/api/2.1/unity-catalog", api_routes)
        // Tag Policies (governed tag definitions) live under /api/2.1, not /unity-catalog.
        .nest("/api/2.1", create_tag_policies_router(handler.clone()))
        .nest(
            "/api/v1/delta-sharing",
            create_sharing_router(handler.clone()),
        )
        // Open Sharing: superset surface sharing the same tabular handlers.
        .nest(
            "/api/v1/open-sharing",
            create_open_sharing_router(handler.clone()),
        );
    let server = router.layer(AuthenticationLayer::new(authenticator));

    run(
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

pub(crate) async fn run<S: Into<String> + Clone>(
    router: axum::Router,
    host: impl AsRef<str>,
    port: u16,
    apis: Vec<ApiDefinition<S>>,
) -> Result<()> {
    let router = apis.into_iter().fold(router, |router, api| {
        router.merge(swagger_ui_dist::generate_routes(api))
    });
    let router = router.layer(
        TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().include_headers(true))
            .on_request(DefaultOnRequest::new().level(Level::INFO))
            .on_response(
                DefaultOnResponse::new()
                    .level(Level::INFO)
                    .latency_unit(LatencyUnit::Micros),
            ),
    );
    let listener = TcpListener::bind(format!("{}:{}", host.as_ref(), port))
        .await
        .map_err(|e| Error::Generic(e.to_string()))?;
    let addr = listener
        .local_addr()
        .map_err(|e| Error::Generic(e.to_string()))?;
    crate::render::status::success(&format!("listening on http://{addr}"));
    tracing::info!("Listening on: {addr}");
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| Error::Generic(e.to_string()))?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
