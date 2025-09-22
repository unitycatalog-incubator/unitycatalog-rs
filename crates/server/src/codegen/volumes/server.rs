#![allow(unused_mut)]
use super::handler::VolumeHandler;
use crate::Result;
use crate::api::RequestContext;
use crate::policy::Principal;
use axum::extract::{Extension, State};
use unitycatalog_common::models::volumes::v1::*;
pub async fn list_volumes<T: VolumeHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Principal>,
    request: ListVolumesRequest,
) -> Result<::axum::Json<ListVolumesResponse>> {
    let context = RequestContext { recipient };
    let result = handler.list_volumes(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_volume<T: VolumeHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Principal>,
    request: CreateVolumeRequest,
) -> Result<::axum::Json<VolumeInfo>> {
    let context = RequestContext { recipient };
    let result = handler.create_volume(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_volume<T: VolumeHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Principal>,
    request: GetVolumeRequest,
) -> Result<::axum::Json<VolumeInfo>> {
    let context = RequestContext { recipient };
    let result = handler.get_volume(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_volume<T: VolumeHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Principal>,
    request: UpdateVolumeRequest,
) -> Result<::axum::Json<VolumeInfo>> {
    let context = RequestContext { recipient };
    let result = handler.update_volume(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_volume<T: VolumeHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Principal>,
    request: DeleteVolumeRequest,
) -> Result<()> {
    let context = RequestContext { recipient };
    handler.delete_volume(request, context).await?;
    Ok(())
}
