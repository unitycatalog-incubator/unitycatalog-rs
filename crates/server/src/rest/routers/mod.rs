use crate::api::{
    CatalogHandler, CredentialHandler, ExternalLocationHandler, RecipientHandler, SchemaHandler,
    ShareHandler, TableHandler, TemporaryCredentialHandler,
};

pub use sharing::get_router as create_sharing_router;

mod sharing;

pub fn create_catalogs_router<T: CatalogHandler + Clone>(handler: T) -> axum::Router {
    use crate::codegen::catalogs::server::*;

    axum::Router::new()
        .route("/catalogs", axum::routing::get(list_catalogs::<T>))
        .route("/catalogs", axum::routing::post(create_catalog::<T>))
        .route("/catalogs/{name}", axum::routing::get(get_catalog::<T>))
        .route(
            "/catalogs/{name}",
            axum::routing::patch(update_catalog::<T>),
        )
        .route(
            "/catalogs/{name}",
            axum::routing::delete(delete_catalog::<T>),
        )
        .with_state(handler)
}

pub fn create_credentials_router<T: CredentialHandler + Clone>(handler: T) -> axum::Router {
    use crate::codegen::credentials::server::*;

    axum::Router::new()
        .route("/credentials", axum::routing::get(list_credentials::<T>))
        .route("/credentials", axum::routing::post(create_credential::<T>))
        .route(
            "/credentials/{name}",
            axum::routing::get(get_credential::<T>),
        )
        .route(
            "/credentials/{name}",
            axum::routing::patch(update_credential::<T>),
        )
        .route(
            "/credentials/{name}",
            axum::routing::delete(delete_credential::<T>),
        )
        .with_state(handler)
}

pub fn create_external_locations_router<T: ExternalLocationHandler + Clone>(
    handler: T,
) -> axum::Router {
    use crate::codegen::external_locations::server::*;

    axum::Router::new()
        .route(
            "/external-locations",
            axum::routing::get(list_external_locations::<T>),
        )
        .route(
            "/external-locations",
            axum::routing::post(create_external_location::<T>),
        )
        .route(
            "/external-locations/{name}",
            axum::routing::get(get_external_location::<T>),
        )
        .route(
            "/external-locations/{name}",
            axum::routing::patch(update_external_location::<T>),
        )
        .route(
            "/external-locations/{name}",
            axum::routing::delete(delete_external_location::<T>),
        )
        .with_state(handler)
}

pub fn create_recipients_router<T: RecipientHandler + Clone>(handler: T) -> axum::Router {
    use crate::codegen::recipients::server::*;

    axum::Router::new()
        .route("/recipients", axum::routing::get(list_recipients::<T>))
        .route("/recipients", axum::routing::post(create_recipient::<T>))
        .route("/recipients/{name}", axum::routing::get(get_recipient::<T>))
        .route(
            "/recipients/{name}",
            axum::routing::patch(update_recipient::<T>),
        )
        .route(
            "/recipients/{name}",
            axum::routing::delete(delete_recipient::<T>),
        )
        .with_state(handler)
}

pub fn create_schemas_router<T: SchemaHandler + Clone>(handler: T) -> axum::Router {
    use crate::codegen::schemas::server::*;

    axum::Router::new()
        .route("/schemas", axum::routing::get(list_schemas::<T>))
        .route("/schemas", axum::routing::post(create_schema::<T>))
        .route("/schemas/{name}", axum::routing::get(get_schema::<T>))
        .route("/schemas/{name}", axum::routing::patch(update_schema::<T>))
        .route("/schemas/{name}", axum::routing::delete(delete_schema::<T>))
        .with_state(handler)
}

pub fn create_shares_router<T: ShareHandler + Clone>(handler: T) -> axum::Router {
    use crate::codegen::shares::server::*;

    axum::Router::new()
        .route("/shares", axum::routing::get(list_shares::<T>))
        .route("/shares", axum::routing::post(create_share::<T>))
        .route("/shares/{name}", axum::routing::get(get_share::<T>))
        .route("/shares/{name}", axum::routing::patch(update_share::<T>))
        .route("/shares/{name}", axum::routing::delete(delete_share::<T>))
        .with_state(handler)
}

pub fn create_tables_router<T: TableHandler + Clone>(handler: T) -> axum::Router {
    use crate::codegen::tables::server::*;

    axum::Router::new()
        .route(
            "/table-summaries",
            axum::routing::get(list_table_summaries::<T>),
        )
        .route("/tables", axum::routing::get(list_tables::<T>))
        .route("/tables", axum::routing::post(create_table::<T>))
        .route("/tables/{full_name}", axum::routing::get(get_table::<T>))
        .route(
            "/tables/{full_name}/exists",
            axum::routing::get(get_table_exists::<T>),
        )
        .route(
            "/tables/{full_name}",
            axum::routing::delete(delete_table::<T>),
        )
        .with_state(handler)
}

pub fn create_temporary_credentials_router<T: TemporaryCredentialHandler + Clone>(
    handler: T,
) -> axum::Router {
    use crate::codegen::temporary_credentials::server::*;

    axum::Router::new()
        .route(
            "/temporary-table-credentials",
            axum::routing::post(generate_temporary_table_credentials::<T>),
        )
        .route(
            "/temporary-path-credentials",
            axum::routing::post(generate_temporary_path_credentials::<T>),
        )
        .with_state(handler)
}
