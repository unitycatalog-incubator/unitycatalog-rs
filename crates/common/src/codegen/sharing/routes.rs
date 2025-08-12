use super::handler::SharingHandler;
use crate::api::RequestContext;
use crate::models::sharing::v1::*;
use crate::services::Recipient;
use crate::Result;
pub async fn list_shares_handler<T: SharingHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: ListSharesRequest,
) -> Result<::axum::Json<ListSharesResponse>> {
    let context = RequestContext { recipient };
    let result = handler.list_shares(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_share_handler<T: SharingHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: GetShareRequest,
) -> Result<::axum::Json<Share>> {
    let context = RequestContext { recipient };
    let result = handler.get_share(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn list_sharing_schemas_handler<T: SharingHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: ListSharingSchemasRequest,
) -> Result<::axum::Json<ListSharingSchemasResponse>> {
    let context = RequestContext { recipient };
    let result = handler.list_sharing_schemas(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn list_schema_tables_handler<T: SharingHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: ListSchemaTablesRequest,
) -> Result<::axum::Json<ListSchemaTablesResponse>> {
    let context = RequestContext { recipient };
    let result = handler.list_schema_tables(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn list_share_tables_handler<T: SharingHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: ListShareTablesRequest,
) -> Result<::axum::Json<ListShareTablesResponse>> {
    let context = RequestContext { recipient };
    let result = handler.list_share_tables(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_table_version_handler<T: SharingHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: GetTableVersionRequest,
) -> Result<::axum::Json<GetTableVersionResponse>> {
    let context = RequestContext { recipient };
    let result = handler.get_table_version(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_table_metadata_handler<T: SharingHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: GetTableMetadataRequest,
) -> Result<::axum::Json<QueryResponse>> {
    let context = RequestContext { recipient };
    let result = handler.get_table_metadata(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn query_table_handler<T: SharingHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: QueryTableRequest,
) -> Result<::axum::Json<QueryResponse>> {
    let context = RequestContext { recipient };
    let result = handler.query_table(request, context).await?;
    Ok(axum::Json(result))
}
