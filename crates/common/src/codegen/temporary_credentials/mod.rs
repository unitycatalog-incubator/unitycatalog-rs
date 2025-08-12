pub mod client;
mod extractors;
mod handler;
pub mod routes;
pub use client::*;
pub use handler::TemporaryCredentialHandler;
use routes::*;
/// Create router for this service
pub fn create_router<T: TemporaryCredentialHandler + Clone>(handler: T) -> axum::Router {
    axum::Router::new()
        .route(
            "/temporary-table-credentials",
            axum::routing::post(generate_temporary_table_credentials_handler::<T>),
        )
        .route(
            "/temporary-volume-credentials",
            axum::routing::post(generate_temporary_volume_credentials_handler::<T>),
        )
        .with_state(handler)
}
