pub mod client;
pub mod extractors;
pub mod handler;
pub mod routes;
pub use handler::ShareHandler;
pub use routes::*;
/// Create router for this service
pub fn create_router<T: ShareHandler + Clone>() -> axum::Router<T> {
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
}
