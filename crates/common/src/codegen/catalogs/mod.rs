pub mod client;
pub mod extractors;
pub mod handler;
pub mod routes;
pub use handler::CatalogHandler;
pub use routes::*;
/// Create router for this service
pub fn create_router<T: CatalogHandler + Clone>() -> axum::Router<T> {
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
}
