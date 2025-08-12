pub mod client;
pub mod extractors;
pub mod handler;
pub mod routes;
pub use handler::TemporaryCredentialHandler;
pub use routes::*;
/// Create router for this service
pub fn create_router<T: TemporaryCredentialHandler + Clone>() -> axum::Router<T> {
    axum::Router::new()
        .route(
            "/temporary-table-credentials",
            axum::routing::post(generate_temporary_table_credentials_handler::<T>),
        )
        .route(
            "/temporary-volume-credentials",
            axum::routing::post(generate_temporary_volume_credentials_handler::<T>),
        )
}
