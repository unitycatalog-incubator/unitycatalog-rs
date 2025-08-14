use unitycatalog_common::api::{
    CatalogHandler, CredentialHandler, ExternalLocationHandler, RecipientHandler, SchemaHandler,
    ShareHandler, TableHandler, TemporaryCredentialHandler,
};

pub use sharing::get_router as create_sharing_router;

mod sharing;

pub fn create_catalogs_router<T: CatalogHandler + Clone>(handler: T) -> axum::Router {
    use unitycatalog_common::api::codegen::catalogs::server::*;

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

pub fn create_credentials_router<T: CredentialHandler + Clone>(handler: T) -> axum::Router {
    use unitycatalog_common::api::codegen::credentials::server::*;

    axum::Router::new()
        .route(
            "/credentials",
            axum::routing::get(list_credentials_handler::<T>),
        )
        .route(
            "/credentials",
            axum::routing::post(create_credential_handler::<T>),
        )
        .route(
            "/credentials/{name}",
            axum::routing::get(get_credential_handler::<T>),
        )
        .route(
            "/credentials/{name}",
            axum::routing::patch(update_credential_handler::<T>),
        )
        .route(
            "/credentials/{name}",
            axum::routing::delete(delete_credential_handler::<T>),
        )
        .with_state(handler)
}

pub fn create_external_locations_router<T: ExternalLocationHandler + Clone>(
    handler: T,
) -> axum::Router {
    use unitycatalog_common::api::codegen::external_locations::server::*;

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

pub fn create_recipients_router<T: RecipientHandler + Clone>(handler: T) -> axum::Router {
    use unitycatalog_common::api::codegen::recipients::server::*;

    axum::Router::new()
        .route(
            "/recipients",
            axum::routing::get(list_recipients_handler::<T>),
        )
        .route(
            "/recipients",
            axum::routing::post(create_recipient_handler::<T>),
        )
        .route(
            "/recipients/{name}",
            axum::routing::get(get_recipient_handler::<T>),
        )
        .route(
            "/recipients/{name}",
            axum::routing::patch(update_recipient_handler::<T>),
        )
        .route(
            "/recipients/{name}",
            axum::routing::delete(delete_recipient_handler::<T>),
        )
        .with_state(handler)
}

pub fn create_schemas_router<T: SchemaHandler + Clone>(handler: T) -> axum::Router {
    use unitycatalog_common::api::codegen::schemas::server::*;

    axum::Router::new()
        .route("/schemas", axum::routing::get(list_schemas_handler::<T>))
        .route("/schemas", axum::routing::post(create_schema_handler::<T>))
        .route(
            "/schemas/{name}",
            axum::routing::get(get_schema_handler::<T>),
        )
        .route(
            "/schemas/{name}",
            axum::routing::patch(update_schema_handler::<T>),
        )
        .route(
            "/schemas/{name}",
            axum::routing::delete(delete_schema_handler::<T>),
        )
        .with_state(handler)
}

pub fn create_shares_router<T: ShareHandler + Clone>(handler: T) -> axum::Router {
    use unitycatalog_common::api::codegen::shares::server::*;

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
        .with_state(handler)
}

pub fn create_tables_router<T: TableHandler + Clone>(handler: T) -> axum::Router {
    use unitycatalog_common::api::codegen::tables::server::*;

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

pub fn create_temporary_credentials_router<T: TemporaryCredentialHandler + Clone>(
    handler: T,
) -> axum::Router {
    use unitycatalog_common::api::codegen::temporary_credentials::server::*;

    axum::Router::new()
        .route(
            "/temporary-table-credentials",
            axum::routing::post(generate_temporary_table_credentials_handler::<T>),
        )
        .route(
            "/temporary-volume-credentials",
            axum::routing::post(generate_temporary_volume_credentials_handler::<T>),
        )
        .with_state(handler)
}
