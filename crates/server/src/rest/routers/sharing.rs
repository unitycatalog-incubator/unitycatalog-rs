use axum::body::Body;
use axum::extract::State;
use axum::response::Response;
use axum::routing::{Router, get, post};
use http::header::CONTENT_TYPE;

use unitycatalog_sharing_client::models::sharing::v1::*;

use crate::api::sharing::{SharingHandler, SharingQueryHandler};
use crate::{Error, Result};

/// Create a new [Router] for the Delta Sharing REST API.
pub fn get_router<T, Cx>(state: T) -> Router
where
    T: SharingHandler<Cx> + SharingQueryHandler<Cx> + Clone,
    Cx: axum::extract::FromRequestParts<T> + Send + 'static,
{
    Router::new()
        .route("/shares", get(list_shares::<T, Cx>))
        .route("/shares/{share}", get(get_share::<T, Cx>))
        .route("/shares/{share}/schemas", get(list_schemas::<T, Cx>))
        .route("/shares/{share}/all-tables", get(list_all_tables::<T, Cx>))
        .route(
            "/shares/{share}/schemas/{name}/tables",
            get(list_tables::<T, Cx>),
        )
        .route(
            "/shares/{share}/schemas/{schema}/tables/{name}/version",
            get(get_table_version::<T, Cx>),
        )
        .route(
            "/shares/{share}/schemas/{schema}/tables/{name}/metadata",
            get(get_table_metadata::<T, Cx>),
        )
        .route(
            "/shares/{share}/schemas/{schema}/tables/{name}/query",
            post(get_table_query::<T, Cx>),
        )
        .with_state(state)
}

pub async fn list_shares<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListSharesRequest,
) -> Result<::axum::Json<ListSharesResponse>>
where
    T: SharingHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.list_shares(request, context).await?;
    Ok(axum::Json(result))
}

pub async fn get_share<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GetShareRequest,
) -> Result<::axum::Json<Share>>
where
    T: SharingHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.get_share(request, context).await?;
    Ok(axum::Json(result))
}

pub async fn list_schemas<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListSchemasRequest,
) -> Result<::axum::Json<ListSchemasResponse>>
where
    T: SharingHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.list_sharing_schemas(request, context).await?;
    Ok(axum::Json(result))
}

pub async fn list_tables<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListTablesRequest,
) -> Result<::axum::Json<ListTablesResponse>>
where
    T: SharingHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.list_tables(request, context).await?;
    Ok(axum::Json(result))
}

pub async fn list_all_tables<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListAllTablesRequest,
) -> Result<::axum::Json<ListAllTablesResponse>>
where
    T: SharingHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.list_all_tables(request, context).await?;
    Ok(axum::Json(result))
}

async fn get_table_version<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GetTableVersionRequest,
) -> Result<Response>
where
    T: SharingQueryHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.get_table_version(request, context).await?;
    Response::builder()
        .header("Delta-Table-Version", result.version)
        .body(Body::empty())
        .map_err(|e| Error::generic(e.to_string()))
}

async fn get_table_metadata<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GetTableMetadataRequest,
) -> Result<Response>
where
    T: SharingQueryHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.get_table_metadata(request, context).await?;
    let response = Response::builder()
        .header(CONTENT_TYPE, "application/x-ndjson; charset=utf-8")
        .body(Body::from(result))
        .map_err(|e| Error::generic(e.to_string()))?;
    Ok(response)
}

async fn get_table_query<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: QueryTableRequest,
) -> Result<Response>
where
    T: SharingQueryHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.query_table(request, context).await?;
    let response = Response::builder()
        .header(CONTENT_TYPE, "application/x-ndjson; charset=utf-8")
        .body(Body::from(result))
        .map_err(|e| Error::generic(e.to_string()))?;
    Ok(response)
}
