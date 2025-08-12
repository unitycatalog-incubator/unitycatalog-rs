pub mod client;
pub mod extractors;
pub mod handler;
pub mod routes;
pub use handler::SchemaHandler;
pub use routes::*;
/// Create router for this service
pub fn create_router<T: SchemaHandler + Clone>() -> axum::Router<T> {
    axum::Router::new()
        .route("/schemas", axum::routing::get(list_schemas_handler::<T>))
        .route("/schemas", axum::routing::post(create_schema_handler::<T>))
        .route(
            "/schemas/{name}",
            axum::routing::get(get_schema_handler::<T>),
        )
        .route(
            "/schemas/{full_name}",
            axum::routing::patch(update_schema_handler::<T>),
        )
        .route(
            "/schemas/{name}",
            axum::routing::delete(delete_schema_handler::<T>),
        )
}
