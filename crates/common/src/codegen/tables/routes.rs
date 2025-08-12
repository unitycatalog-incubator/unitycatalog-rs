use super::handler::TableHandler;
use crate::api::RequestContext;
use crate::models::tables::v1::*;
use crate::services::Recipient;
use crate::Result;
pub async fn list_table_summaries_handler<T: TableHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: ListTableSummariesRequest,
) -> Result<::axum::Json<ListTableSummariesResponse>> {
    let context = RequestContext { recipient };
    let result = handler.list_table_summaries(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn list_tables_handler<T: TableHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: ListTablesRequest,
) -> Result<::axum::Json<ListTablesResponse>> {
    let context = RequestContext { recipient };
    let result = handler.list_tables(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_table_handler<T: TableHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: CreateTableRequest,
) -> Result<::axum::Json<TableInfo>> {
    let context = RequestContext { recipient };
    let result = handler.create_table(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_table_handler<T: TableHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: GetTableRequest,
) -> Result<::axum::Json<TableInfo>> {
    let context = RequestContext { recipient };
    let result = handler.get_table(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_table_exists_handler<T: TableHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: GetTableExistsRequest,
) -> Result<::axum::Json<GetTableExistsResponse>> {
    let context = RequestContext { recipient };
    let result = handler.get_table_exists(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_table_handler<T: TableHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: DeleteTableRequest,
) -> Result<()> {
    let context = RequestContext { recipient };
    handler.delete_table(request, context).await?;
    Ok(())
}
