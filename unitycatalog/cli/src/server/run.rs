use axum::Router;
use swagger_ui_dist::{ApiDefinition, OpenApiSource};
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::LatencyUnit;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;
use unitycatalog_common::api::catalogs::CatalogHandler;
use unitycatalog_common::api::credentials::CredentialsHandler;
use unitycatalog_common::api::external_locations::ExternalLocationsHandler;
use unitycatalog_common::api::recipients::RecipientsHandler;
use unitycatalog_common::api::schemas::SchemasHandler;
use unitycatalog_common::api::shares::SharesHandler;
use unitycatalog_common::api::sharing::{SharingDiscoveryHandler, SharingQueryHandler};
use unitycatalog_common::api::tables::TablesHandler;
use unitycatalog_common::rest::{
    AuthenticationLayer, Authenticator, get_catalog_router, get_credentials_router,
    get_external_locations_router, get_recipients_router, get_schemas_router, get_shares_router,
    get_sharing_router, get_tables_router,
};
use unitycatalog_common::{Error, Result};

pub async fn run_server_rest<T, A>(
    host: impl AsRef<str>,
    port: u16,
    handler: T,
    authenticator: A,
) -> Result<()>
where
    T: CatalogHandler
        + CredentialsHandler
        + SharingDiscoveryHandler
        + SharingQueryHandler
        + SharesHandler
        + SchemasHandler
        + TablesHandler
        + ExternalLocationsHandler
        + RecipientsHandler
        + Clone,
    A: Authenticator + Clone,
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

    let api_routes = get_catalog_router(handler.clone())
        .merge(get_schemas_router(handler.clone()))
        .merge(get_tables_router(handler.clone()))
        .merge(get_credentials_router(handler.clone()))
        .merge(get_external_locations_router(handler.clone()))
        .merge(get_recipients_router(handler.clone()))
        .merge(get_shares_router(handler.clone()));

    let router = Router::new()
        .nest("/api/2.1/unity-catalog", api_routes)
        .nest("/api/v1/delta-sharing", get_sharing_router(handler.clone()));
    let server = router.layer(AuthenticationLayer::new(authenticator));

    run(server, host, port, api_def, sharing_api_def).await
}

async fn run<S: Into<String> + Clone>(
    router: axum::Router,
    host: impl AsRef<str>,
    port: u16,
    api: ApiDefinition<S>,
    sharing_api: ApiDefinition<S>,
) -> Result<()> {
    let router = router
        .merge(swagger_ui_dist::generate_routes(api))
        .merge(swagger_ui_dist::generate_routes(sharing_api));
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
    tracing::info!("Listning on: {}", listener.local_addr().unwrap());
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
