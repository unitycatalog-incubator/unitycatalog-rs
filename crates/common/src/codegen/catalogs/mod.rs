pub mod client;
mod extractors;
mod handler;
pub mod routes;
pub use client::*;
pub use handler::CatalogHandler;
use routes::*;
/// Create router for this service
pub fn create_router<T: CatalogHandler + Clone>(handler: T) -> axum::Router {
    axum::Router::new()
        .route("/catalogs", axum::routing::get(list_catalogs_handler::<T>))
        .route(
            "/catalogs",
            axum::routing::post(create_catalog_handler::<T>),
        )
        .route(
            "/catalogs/{name}",
            axum::routing::get(get_catalog_handler::<T>),
        )
        .route(
            "/catalogs/{name}",
            axum::routing::patch(update_catalog_handler::<T>),
        )
        .route(
            "/catalogs/{name}",
            axum::routing::delete(delete_catalog_handler::<T>),
        )
        .with_state(handler)
}
