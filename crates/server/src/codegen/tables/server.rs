#![allow(unused_mut)]
use super::handler::TableHandler;
use crate::Result;
use crate::api::RequestContext;
use crate::policy::Principal;
use axum::extract::{Extension, State};
use unitycatalog_common::models::tables::v1::*;
pub async fn list_table_summaries<T: TableHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Principal>,
    request: ListTableSummariesRequest,
) -> Result<::axum::Json<ListTableSummariesResponse>> {
    let context = RequestContext { recipient };
    let result = handler.list_table_summaries(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn list_tables<T: TableHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Principal>,
    request: ListTablesRequest,
) -> Result<::axum::Json<ListTablesResponse>> {
    let context = RequestContext { recipient };
    let result = handler.list_tables(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_table<T: TableHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Principal>,
    request: CreateTableRequest,
) -> Result<::axum::Json<Table>> {
    let context = RequestContext { recipient };
    let result = handler.create_table(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_table<T: TableHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Principal>,
    request: GetTableRequest,
) -> Result<::axum::Json<Table>> {
    let context = RequestContext { recipient };
    let result = handler.get_table(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_table_exists<T: TableHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Principal>,
    request: GetTableExistsRequest,
) -> Result<::axum::Json<GetTableExistsResponse>> {
    let context = RequestContext { recipient };
    let result = handler.get_table_exists(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_table<T: TableHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Principal>,
    request: DeleteTableRequest,
) -> Result<()> {
    let context = RequestContext { recipient };
    handler.delete_table(request, context).await?;
    Ok(())
}
