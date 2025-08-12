pub mod client;
mod extractors;
mod handler;
pub mod routes;
pub use client::*;
pub use handler::ShareHandler;
use routes::*;
/// Create router for this service
pub fn create_router<T: ShareHandler + Clone>(handler: T) -> axum::Router {
    axum::Router::new()
        .route("/shares", axum::routing::get(list_shares_handler::<T>))
        .route("/shares", axum::routing::post(create_share_handler::<T>))
        .route("/shares/{name}", axum::routing::get(get_share_handler::<T>))
        .route(
            "/shares/{name}",
            axum::routing::patch(update_share_handler::<T>),
        )
        .route(
            "/shares/{name}",
            axum::routing::delete(delete_share_handler::<T>),
        )
        .with_state(handler)
}
