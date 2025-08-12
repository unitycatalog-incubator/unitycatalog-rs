pub mod client;
pub mod extractors;
pub mod handler;
pub mod routes;
pub use handler::TableHandler;
pub use routes::*;
/// Create router for this service
pub fn create_router<T: TableHandler + Clone>() -> axum::Router<T> {
    axum::Router::new()
        .route(
            "/table-summaries",
            axum::routing::get(list_table_summaries_handler::<T>),
        )
        .route("/tables", axum::routing::get(list_tables_handler::<T>))
        .route("/tables", axum::routing::post(create_table_handler::<T>))
        .route(
            "/tables/{full_name}",
            axum::routing::get(get_table_handler::<T>),
        )
        .route(
            "/tables/{full_name}/exists",
            axum::routing::get(get_table_exists_handler::<T>),
        )
        .route(
            "/tables/{full_name}",
            axum::routing::delete(delete_table_handler::<T>),
        )
}
