use super::handler::TableHandler;
use crate::Result;
use crate::api::RequestContext;
use crate::models::tables::v1::*;
use crate::services::Recipient;
use axum::extract::{Extension, State};
use axum::{RequestExt, RequestPartsExt};
pub async fn list_table_summaries_handler<T: TableHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: ListTableSummariesRequest,
) -> Result<::axum::Json<ListTableSummariesResponse>> {
    let context = RequestContext { recipient };
    let result = handler.list_table_summaries(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn list_tables_handler<T: TableHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: ListTablesRequest,
) -> Result<::axum::Json<ListTablesResponse>> {
    let context = RequestContext { recipient };
    let result = handler.list_tables(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_table_handler<T: TableHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: CreateTableRequest,
) -> Result<::axum::Json<TableInfo>> {
    let context = RequestContext { recipient };
    let result = handler.create_table(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_table_handler<T: TableHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: GetTableRequest,
) -> Result<::axum::Json<TableInfo>> {
    let context = RequestContext { recipient };
    let result = handler.get_table(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_table_exists_handler<T: TableHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: GetTableExistsRequest,
) -> Result<::axum::Json<GetTableExistsResponse>> {
    let context = RequestContext { recipient };
    let result = handler.get_table_exists(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_table_handler<T: TableHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: DeleteTableRequest,
) -> Result<()> {
    let context = RequestContext { recipient };
    handler.delete_table(request, context).await?;
    Ok(())
}
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for ListTableSummariesRequest {
    type Rejection = crate::Error;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        #[derive(serde::Deserialize)]
        struct QueryParams {
            catalog_name: String,
            #[serde(default)]
            schema_name_pattern: Option<String>,
            #[serde(default)]
            table_name_pattern: Option<String>,
            #[serde(default)]
            max_results: Option<i32>,
            #[serde(default)]
            page_token: Option<String>,
            #[serde(default)]
            include_manifest_capabilities: Option<bool>,
        }
        let axum::extract::Query(QueryParams {
            catalog_name,
            schema_name_pattern,
            table_name_pattern,
            max_results,
            page_token,
            include_manifest_capabilities,
        }) = parts.extract::<axum::extract::Query<QueryParams>>().await?;
        Ok(ListTableSummariesRequest {
            catalog_name,
            schema_name_pattern,
            table_name_pattern,
            max_results,
            page_token,
            include_manifest_capabilities,
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for ListTablesRequest {
    type Rejection = crate::Error;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        #[derive(serde::Deserialize)]
        struct QueryParams {
            schema_name: String,
            catalog_name: String,
            #[serde(default)]
            max_results: Option<i32>,
            #[serde(default)]
            page_token: Option<String>,
            #[serde(default)]
            include_delta_metadata: Option<bool>,
            #[serde(default)]
            omit_columns: Option<bool>,
            #[serde(default)]
            omit_properties: Option<bool>,
            #[serde(default)]
            omit_username: Option<bool>,
            #[serde(default)]
            include_browse: Option<bool>,
            #[serde(default)]
            include_manifest_capabilities: Option<bool>,
        }
        let axum::extract::Query(QueryParams {
            schema_name,
            catalog_name,
            max_results,
            page_token,
            include_delta_metadata,
            omit_columns,
            omit_properties,
            omit_username,
            include_browse,
            include_manifest_capabilities,
        }) = parts.extract::<axum::extract::Query<QueryParams>>().await?;
        Ok(ListTablesRequest {
            schema_name,
            catalog_name,
            max_results,
            page_token,
            include_delta_metadata,
            omit_columns,
            omit_properties,
            omit_username,
            include_browse,
            include_manifest_capabilities,
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequest<S> for CreateTableRequest {
    type Rejection = axum::response::Response;
    async fn from_request(
        req: axum::extract::Request<axum::body::Body>,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Json(request) = req
            .extract()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(request)
    }
}
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for GetTableRequest {
    type Rejection = crate::Error;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path((full_name)) =
            parts.extract::<axum::extract::Path<(String)>>().await?;
        #[derive(serde::Deserialize)]
        struct QueryParams {
            #[serde(default)]
            include_delta_metadata: Option<bool>,
            #[serde(default)]
            include_browse: Option<bool>,
            #[serde(default)]
            include_manifest_capabilities: Option<bool>,
        }
        let axum::extract::Query(QueryParams {
            include_delta_metadata,
            include_browse,
            include_manifest_capabilities,
        }) = parts.extract::<axum::extract::Query<QueryParams>>().await?;
        Ok(GetTableRequest {
            full_name,
            include_delta_metadata,
            include_browse,
            include_manifest_capabilities,
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for GetTableExistsRequest {
    type Rejection = crate::Error;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path((full_name)) =
            parts.extract::<axum::extract::Path<(String)>>().await?;
        Ok(GetTableExistsRequest { full_name })
    }
}
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for DeleteTableRequest {
    type Rejection = crate::Error;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path((full_name)) =
            parts.extract::<axum::extract::Path<(String)>>().await?;
        Ok(DeleteTableRequest { full_name })
    }
}
