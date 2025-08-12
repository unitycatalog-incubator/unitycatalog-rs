pub mod client;
pub mod extractors;
pub mod handler;
pub mod routes;
pub use handler::RecipientHandler;
pub use routes::*;
/// Create router for this service
pub fn create_router<T: RecipientHandler + Clone>() -> axum::Router<T> {
    axum::Router::new()
        .route(
            "/recipients",
            axum::routing::get(list_recipients_handler::<T>),
        )
        .route(
            "/recipients",
            axum::routing::post(create_recipient_handler::<T>),
        )
        .route(
            "/recipients/{name}",
            axum::routing::get(get_recipient_handler::<T>),
        )
        .route(
            "/recipients/{name}",
            axum::routing::patch(update_recipient_handler::<T>),
        )
        .route(
            "/recipients/{name}",
            axum::routing::delete(delete_recipient_handler::<T>),
        )
}
