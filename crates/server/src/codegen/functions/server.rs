// @generated — do not edit by hand.
#![allow(unused_mut)]
use super::handler::FunctionHandler;
use crate::Result;
use crate::api::RequestContext;
use crate::policy::Principal;
use axum::extract::{Extension, State};
use unitycatalog_common::models::functions::v1::*;
pub async fn list_functions<T: FunctionHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Principal>,
    request: ListFunctionsRequest,
) -> Result<::axum::Json<ListFunctionsResponse>> {
    let context = RequestContext { recipient };
    let result = handler.list_functions(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_function<T: FunctionHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Principal>,
    request: CreateFunctionRequest,
) -> Result<::axum::Json<Function>> {
    let context = RequestContext { recipient };
    let result = handler.create_function(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_function<T: FunctionHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Principal>,
    request: GetFunctionRequest,
) -> Result<::axum::Json<Function>> {
    let context = RequestContext { recipient };
    let result = handler.get_function(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_function<T: FunctionHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Principal>,
    request: UpdateFunctionRequest,
) -> Result<::axum::Json<Function>> {
    let context = RequestContext { recipient };
    let result = handler.update_function(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_function<T: FunctionHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Principal>,
    request: DeleteFunctionRequest,
) -> Result<()> {
    let context = RequestContext { recipient };
    handler.delete_function(request, context).await?;
    Ok(())
}
