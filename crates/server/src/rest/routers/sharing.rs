use axum::body::Body;
use axum::extract::{Extension, State};
use axum::response::Response;
use axum::routing::{Router, get, post};
use http::header::CONTENT_TYPE;

use unitycatalog_sharing_client::models::sharing::v1::*;

use crate::api::RequestContext;
use crate::api::sharing::{SharingHandler, SharingQueryHandler};
use crate::policy::Recipient;
use crate::{Error, Result};

/// Create a new [Router] for the Delta Sharing REST API.
pub fn get_router<T: SharingHandler + SharingQueryHandler + Clone>(state: T) -> Router {
    Router::new()
        .route("/shares", get(list_shares::<T>))
        .route("/shares/{share}", get(get_share::<T>))
        .route("/shares/{share}/schemas", get(list_sharing_schemas::<T>))
        .route("/shares/{share}/all-tables", get(list_share_tables::<T>))
        .route(
            "/shares/{share}/schemas/{name}/tables",
            get(list_schema_tables::<T>),
        )
        .route(
            "/shares/{share}/schemas/{schema}/tables/{name}/version",
            get(get_table_version::<T>),
        )
        .route(
            "/shares/{share}/schemas/{schema}/tables/{name}/metadata",
            get(get_table_metadata::<T>),
        )
        .route(
            "/shares/{share}/schemas/{schema}/tables/{name}/query",
            post(get_table_query::<T>),
        )
        .with_state(state)
}

pub async fn list_shares<T: SharingHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: ListSharesRequest,
) -> Result<::axum::Json<ListSharesResponse>> {
    let context = RequestContext { recipient };
    let result = handler.list_shares(request, context).await?;
    Ok(axum::Json(result))
}

pub async fn get_share<T: SharingHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: GetShareRequest,
) -> Result<::axum::Json<Share>> {
    let context = RequestContext { recipient };
    let result = handler.get_share(request, context).await?;
    Ok(axum::Json(result))
}

pub async fn list_sharing_schemas<T: SharingHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: ListSharingSchemasRequest,
) -> Result<::axum::Json<ListSharingSchemasResponse>> {
    let context = RequestContext { recipient };
    let result = handler.list_sharing_schemas(request, context).await?;
    Ok(axum::Json(result))
}

pub async fn list_schema_tables<T: SharingHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: ListSchemaTablesRequest,
) -> Result<::axum::Json<ListSchemaTablesResponse>> {
    let context = RequestContext { recipient };
    let result = handler.list_schema_tables(request, context).await?;
    Ok(axum::Json(result))
}

pub async fn list_share_tables<T: SharingHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: ListShareTablesRequest,
) -> Result<::axum::Json<ListShareTablesResponse>> {
    let context = RequestContext { recipient };
    let result = handler.list_share_tables(request, context).await?;
    Ok(axum::Json(result))
}

async fn get_table_version<T: SharingQueryHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: GetTableVersionRequest,
) -> Result<Response> {
    let ctx = RequestContext { recipient };
    let result = handler.get_table_version(request, ctx).await?;
    Response::builder()
        .header("Delta-Table-Version", result.version)
        .body(Body::empty())
        .map_err(|e| Error::generic(e.to_string()))
}

async fn get_table_metadata<T: SharingQueryHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: GetTableMetadataRequest,
) -> Result<Response> {
    let ctx = RequestContext { recipient };
    let result = handler.get_table_metadata(request, ctx).await?;
    let response = Response::builder()
        .header(CONTENT_TYPE, "application/x-ndjson; charset=utf-8")
        .body(Body::from(result))
        .map_err(|e| Error::generic(e.to_string()))?;
    Ok(response)
}

async fn get_table_query<T: SharingQueryHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: QueryTableRequest,
) -> Result<Response> {
    let ctx = RequestContext { recipient };
    let result = handler.query_table(request, ctx).await?;
    let response = Response::builder()
        .header(CONTENT_TYPE, "application/x-ndjson; charset=utf-8")
        .body(Body::from(result))
        .map_err(|e| Error::generic(e.to_string()))?;
    Ok(response)
}
