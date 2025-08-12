use super::handler::SchemaHandler;
use crate::api::RequestContext;
use crate::models::schemas::v1::*;
use crate::services::Recipient;
use crate::Result;
pub async fn list_schemas_handler<T: SchemaHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: ListSchemasRequest,
) -> Result<::axum::Json<ListSchemasResponse>> {
    let context = RequestContext { recipient };
    let result = handler.list_schemas(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_schema_handler<T: SchemaHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: CreateSchemaRequest,
) -> Result<::axum::Json<SchemaInfo>> {
    let context = RequestContext { recipient };
    let result = handler.create_schema(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_schema_handler<T: SchemaHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: GetSchemaRequest,
) -> Result<::axum::Json<SchemaInfo>> {
    let context = RequestContext { recipient };
    let result = handler.get_schema(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_schema_handler<T: SchemaHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: UpdateSchemaRequest,
) -> Result<::axum::Json<SchemaInfo>> {
    let context = RequestContext { recipient };
    let result = handler.update_schema(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_schema_handler<T: SchemaHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: DeleteSchemaRequest,
) -> Result<()> {
    let context = RequestContext { recipient };
    handler.delete_schema(request, context).await?;
    Ok(())
}
