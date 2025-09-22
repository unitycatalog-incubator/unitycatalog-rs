use crate::api::{
    CatalogHandler, CredentialHandler, ExternalLocationHandler, RecipientHandler, SchemaHandler,
    ShareHandler, TableHandler, TemporaryCredentialHandler, VolumeHandler,
};
use axum::routing::{delete, get, patch, post};

pub use sharing::get_router as create_sharing_router;

mod sharing;

pub fn create_catalogs_router<T: CatalogHandler + Clone>(handler: T) -> axum::Router {
    use crate::codegen::catalogs::server::*;

    axum::Router::new()
        .route("/catalogs", get(list_catalogs::<T>))
        .route("/catalogs", post(create_catalog::<T>))
        .route("/catalogs/{name}", get(get_catalog::<T>))
        .route("/catalogs/{name}", patch(update_catalog::<T>))
        .route("/catalogs/{name}", delete(delete_catalog::<T>))
        .with_state(handler)
}

pub fn create_credentials_router<T: CredentialHandler + Clone>(handler: T) -> axum::Router {
    use crate::codegen::credentials::server::*;

    axum::Router::new()
        .route("/credentials", get(list_credentials::<T>))
        .route("/credentials", post(create_credential::<T>))
        .route("/credentials/{name}", get(get_credential::<T>))
        .route("/credentials/{name}", patch(update_credential::<T>))
        .route("/credentials/{name}", delete(delete_credential::<T>))
        .with_state(handler)
}

pub fn create_external_locations_router<T: ExternalLocationHandler + Clone>(
    handler: T,
) -> axum::Router {
    use crate::codegen::external_locations::server::*;

    axum::Router::new()
        .route("/external-locations", get(list_external_locations::<T>))
        .route("/external-locations", post(create_external_location::<T>))
        .route(
            "/external-locations/{name}",
            get(get_external_location::<T>),
        )
        .route(
            "/external-locations/{name}",
            patch(update_external_location::<T>),
        )
        .route(
            "/external-locations/{name}",
            delete(delete_external_location::<T>),
        )
        .with_state(handler)
}

pub fn create_recipients_router<T: RecipientHandler + Clone>(handler: T) -> axum::Router {
    use crate::codegen::recipients::server::*;

    axum::Router::new()
        .route("/recipients", get(list_recipients::<T>))
        .route("/recipients", post(create_recipient::<T>))
        .route("/recipients/{name}", get(get_recipient::<T>))
        .route("/recipients/{name}", patch(update_recipient::<T>))
        .route("/recipients/{name}", delete(delete_recipient::<T>))
        .with_state(handler)
}

pub fn create_schemas_router<T: SchemaHandler + Clone>(handler: T) -> axum::Router {
    use crate::codegen::schemas::server::*;

    axum::Router::new()
        .route("/schemas", get(list_schemas::<T>))
        .route("/schemas", post(create_schema::<T>))
        .route("/schemas/{name}", get(get_schema::<T>))
        .route("/schemas/{name}", patch(update_schema::<T>))
        .route("/schemas/{name}", delete(delete_schema::<T>))
        .with_state(handler)
}

pub fn create_shares_router<T: ShareHandler + Clone>(handler: T) -> axum::Router {
    use crate::codegen::shares::server::*;

    axum::Router::new()
        .route("/shares", get(list_shares::<T>))
        .route("/shares", post(create_share::<T>))
        .route("/shares/{name}", get(get_share::<T>))
        .route("/shares/{name}", patch(update_share::<T>))
        .route("/shares/{name}", delete(delete_share::<T>))
        .route("/shares/{name}/permissions", get(get_permissions::<T>))
        .route("/shares/{name}/permissions", patch(update_permissions::<T>))
        .with_state(handler)
}

pub fn create_tables_router<T: TableHandler + Clone>(handler: T) -> axum::Router {
    use crate::codegen::tables::server::*;

    axum::Router::new()
        .route("/table-summaries", get(list_table_summaries::<T>))
        .route("/tables", get(list_tables::<T>))
        .route("/tables", post(create_table::<T>))
        .route("/tables/{full_name}", get(get_table::<T>))
        .route("/tables/{full_name}/exists", get(get_table_exists::<T>))
        .route("/tables/{full_name}", delete(delete_table::<T>))
        .with_state(handler)
}

pub fn create_temporary_credentials_router<T: TemporaryCredentialHandler + Clone>(
    handler: T,
) -> axum::Router {
    use crate::codegen::temporary_credentials::server::*;

    axum::Router::new()
        .route(
            "/temporary-table-credentials",
            post(generate_temporary_table_credentials::<T>),
        )
        .route(
            "/temporary-path-credentials",
            post(generate_temporary_path_credentials::<T>),
        )
        .with_state(handler)
}

pub fn create_volumes_router<T: VolumeHandler + Clone>(handler: T) -> axum::Router {
    use crate::codegen::volumes::server::*;

    axum::Router::new()
        .route("/volumes", get(list_volumes::<T>))
        .route("/volumes", post(create_volume::<T>))
        .route("/volumes/{name}", get(get_volume::<T>))
        .route("/volumes/{name}", patch(update_volume::<T>))
        .route("/volumes/{name}", delete(delete_volume::<T>))
        .with_state(handler)
}
