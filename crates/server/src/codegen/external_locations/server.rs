#![allow(unused_mut)]
use super::handler::ExternalLocationHandler;
use crate::Result;
use crate::api::RequestContext;
use crate::policy::Principal;
use axum::extract::{Extension, State};
use unitycatalog_common::models::external_locations::v1::*;
pub async fn list_external_locations<T: ExternalLocationHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Principal>,
    request: ListExternalLocationsRequest,
) -> Result<::axum::Json<ListExternalLocationsResponse>> {
    let context = RequestContext { recipient };
    let result = handler.list_external_locations(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_external_location<T: ExternalLocationHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Principal>,
    request: CreateExternalLocationRequest,
) -> Result<::axum::Json<ExternalLocationInfo>> {
    let context = RequestContext { recipient };
    let result = handler.create_external_location(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_external_location<T: ExternalLocationHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Principal>,
    request: GetExternalLocationRequest,
) -> Result<::axum::Json<ExternalLocationInfo>> {
    let context = RequestContext { recipient };
    let result = handler.get_external_location(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_external_location<T: ExternalLocationHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Principal>,
    request: UpdateExternalLocationRequest,
) -> Result<::axum::Json<ExternalLocationInfo>> {
    let context = RequestContext { recipient };
    let result = handler.update_external_location(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_external_location<T: ExternalLocationHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Principal>,
    request: DeleteExternalLocationRequest,
) -> Result<()> {
    let context = RequestContext { recipient };
    handler.delete_external_location(request, context).await?;
    Ok(())
}
