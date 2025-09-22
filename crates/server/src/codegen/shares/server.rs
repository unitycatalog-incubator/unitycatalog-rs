#![allow(unused_mut)]
use super::handler::ShareHandler;
use crate::Result;
use crate::api::RequestContext;
use crate::policy::Recipient;
use axum::extract::{Extension, State};
use unitycatalog_common::models::shares::v1::*;
pub async fn list_shares<T: ShareHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: ListSharesRequest,
) -> Result<::axum::Json<ListSharesResponse>> {
    let context = RequestContext { recipient };
    let result = handler.list_shares(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_share<T: ShareHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: CreateShareRequest,
) -> Result<::axum::Json<Share>> {
    let context = RequestContext { recipient };
    let result = handler.create_share(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_share<T: ShareHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: GetShareRequest,
) -> Result<::axum::Json<Share>> {
    let context = RequestContext { recipient };
    let result = handler.get_share(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_share<T: ShareHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: UpdateShareRequest,
) -> Result<::axum::Json<Share>> {
    let context = RequestContext { recipient };
    let result = handler.update_share(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_share<T: ShareHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: DeleteShareRequest,
) -> Result<()> {
    let context = RequestContext { recipient };
    handler.delete_share(request, context).await?;
    Ok(())
}
pub async fn get_permissions<T: ShareHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: GetPermissionsRequest,
) -> Result<::axum::Json<GetPermissionsResponse>> {
    let context = RequestContext { recipient };
    let result = handler.get_permissions(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_permissions<T: ShareHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: UpdatePermissionsRequest,
) -> Result<::axum::Json<UpdatePermissionsResponse>> {
    let context = RequestContext { recipient };
    let result = handler.update_permissions(request, context).await?;
    Ok(axum::Json(result))
}
