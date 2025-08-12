pub mod client;
mod extractors;
mod handler;
pub mod routes;
pub use client::*;
pub use handler::ExternalLocationHandler;
use routes::*;
/// Create router for this service
pub fn create_router<T: ExternalLocationHandler + Clone>(handler: T) -> axum::Router {
    axum::Router::new()
        .route(
            "/external-locations",
            axum::routing::get(list_external_locations_handler::<T>),
        )
        .route(
            "/external-locations",
            axum::routing::post(create_external_location_handler::<T>),
        )
        .route(
            "/external-locations/{name}",
            axum::routing::get(get_external_location_handler::<T>),
        )
        .route(
            "/external-locations/{name}",
            axum::routing::patch(update_external_location_handler::<T>),
        )
        .route(
            "/external-locations/{name}",
            axum::routing::delete(delete_external_location_handler::<T>),
        )
        .with_state(handler)
}
