#![allow(unused_mut)]
use super::handler::SchemaHandler;
use crate::Result;
use crate::api::RequestContext;
use crate::policy::Recipient;
use axum::extract::{Extension, State};
use unitycatalog_common::models::schemas::v1::*;
pub async fn list_schemas<T: SchemaHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: ListSchemasRequest,
) -> Result<::axum::Json<ListSchemasResponse>> {
    let context = RequestContext { recipient };
    let result = handler.list_schemas(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_schema<T: SchemaHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: CreateSchemaRequest,
) -> Result<::axum::Json<SchemaInfo>> {
    let context = RequestContext { recipient };
    let result = handler.create_schema(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_schema<T: SchemaHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: GetSchemaRequest,
) -> Result<::axum::Json<SchemaInfo>> {
    let context = RequestContext { recipient };
    let result = handler.get_schema(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_schema<T: SchemaHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: UpdateSchemaRequest,
) -> Result<::axum::Json<SchemaInfo>> {
    let context = RequestContext { recipient };
    let result = handler.update_schema(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_schema<T: SchemaHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: DeleteSchemaRequest,
) -> Result<()> {
    let context = RequestContext { recipient };
    handler.delete_schema(request, context).await?;
    Ok(())
}
