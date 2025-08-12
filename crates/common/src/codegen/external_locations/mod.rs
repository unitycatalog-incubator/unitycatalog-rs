pub mod client;
pub mod extractors;
pub mod handler;
pub mod routes;
pub use handler::ExternalLocationHandler;
pub use routes::*;
/// Create router for this service
pub fn create_router<T: ExternalLocationHandler + Clone>() -> axum::Router<T> {
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
}
