pub mod client;
mod extractors;
mod handler;
pub mod routes;
pub use client::*;
pub use handler::TableHandler;
use routes::*;
/// Create router for this service
pub fn create_router<T: TableHandler + Clone>(handler: T) -> axum::Router {
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
        .with_state(handler)
}
