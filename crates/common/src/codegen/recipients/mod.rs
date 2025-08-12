pub mod client;
mod extractors;
mod handler;
pub mod routes;
pub use client::*;
pub use handler::RecipientHandler;
use routes::*;
/// Create router for this service
pub fn create_router<T: RecipientHandler + Clone>(handler: T) -> axum::Router {
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
        .with_state(handler)
}
