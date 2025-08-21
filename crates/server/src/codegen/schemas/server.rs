#![allow(unused_mut)]
use unitycatalog_common::Result;
use crate::api::RequestContext;
use unitycatalog_common::models::schemas::v1::*;
use super::handler::SchemaHandler;
use crate::policy::Recipient;
use axum::extract::{State, Extension};
pub async fn list_schemas_handler<T: SchemaHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: ListSchemasRequest,
) -> Result<::axum::Json<ListSchemasResponse>> {
    let context = RequestContext { recipient };
    let result = handler.list_schemas(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_schema_handler<T: SchemaHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: CreateSchemaRequest,
) -> Result<::axum::Json<SchemaInfo>> {
    let context = RequestContext { recipient };
    let result = handler.create_schema(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_schema_handler<T: SchemaHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: GetSchemaRequest,
) -> Result<::axum::Json<SchemaInfo>> {
    let context = RequestContext { recipient };
    let result = handler.get_schema(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_schema_handler<T: SchemaHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: UpdateSchemaRequest,
) -> Result<::axum::Json<SchemaInfo>> {
    let context = RequestContext { recipient };
    let result = handler.update_schema(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_schema_handler<T: SchemaHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: DeleteSchemaRequest,
) -> Result<()> {
    let context = RequestContext { recipient };
    handler.delete_schema(request, context).await?;
    Ok(())
}
