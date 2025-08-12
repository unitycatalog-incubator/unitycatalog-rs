pub mod client;
pub mod extractors;
pub mod handler;
pub mod routes;
pub use handler::CredentialHandler;
pub use routes::*;
/// Create router for this service
pub fn create_router<T: CredentialHandler + Clone>() -> axum::Router<T> {
    axum::Router::new()
        .route(
            "/credentials",
            axum::routing::get(list_credentials_handler::<T>),
        )
        .route(
            "/credentials",
            axum::routing::post(create_credential_handler::<T>),
        )
        .route(
            "/credentials/{name}",
            axum::routing::get(get_credential_handler::<T>),
        )
        .route(
            "/credentials/{name}",
            axum::routing::patch(update_credential_handler::<T>),
        )
        .route(
            "/credentials/{name}",
            axum::routing::delete(delete_credential_handler::<T>),
        )
}
