pub mod client;
mod extractors;
mod handler;
pub mod routes;
pub use client::*;
pub use handler::SharingHandler;
use routes::*;
/// Create router for this service
pub fn create_router<T: SharingHandler + Clone>(handler: T) -> axum::Router {
    axum::Router::new()
        .route("/shares", axum::routing::get(list_shares_handler::<T>))
        .route("/shares/{name}", axum::routing::get(get_share_handler::<T>))
        .route(
            "/shares/{share}/schemas",
            axum::routing::get(list_sharing_schemas_handler::<T>),
        )
        .route(
            "/shares/{share}/schemas/{name}/tables",
            axum::routing::get(list_schema_tables_handler::<T>),
        )
        .route(
            "/shares/{name}/all-tables",
            axum::routing::get(list_share_tables_handler::<T>),
        )
        .route(
            "/shares/{share}/schemas/{schema}/tables/{name}/version",
            axum::routing::get(get_table_version_handler::<T>),
        )
        .route(
            "/shares/{share}/schemas/{schema}/tables/{name}/metadata",
            axum::routing::get(get_table_metadata_handler::<T>),
        )
        .route(
            "/shares/{share}/schemas/{schema}/tables/{name}/query",
            axum::routing::post(query_table_handler::<T>),
        )
        .with_state(handler)
}
