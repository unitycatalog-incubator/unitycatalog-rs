#![allow(unused_mut)]
use unitycatalog_common::Result;
use crate::api::RequestContext;
use unitycatalog_common::models::volumes::v1::*;
use super::handler::VolumeHandler;
use crate::policy::Recipient;
use axum::extract::{State, Extension};
pub async fn list_volumes_handler<T: VolumeHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: ListVolumesRequest,
) -> Result<::axum::Json<ListVolumesResponse>> {
    let context = RequestContext { recipient };
    let result = handler.list_volumes(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_volume_handler<T: VolumeHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: CreateVolumeRequest,
) -> Result<::axum::Json<VolumeInfo>> {
    let context = RequestContext { recipient };
    let result = handler.create_volume(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_volume_handler<T: VolumeHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: GetVolumeRequest,
) -> Result<::axum::Json<VolumeInfo>> {
    let context = RequestContext { recipient };
    let result = handler.get_volume(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_volume_handler<T: VolumeHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: UpdateVolumeRequest,
) -> Result<::axum::Json<VolumeInfo>> {
    let context = RequestContext { recipient };
    let result = handler.update_volume(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_volume_handler<T: VolumeHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: DeleteVolumeRequest,
) -> Result<()> {
    let context = RequestContext { recipient };
    handler.delete_volume(request, context).await?;
    Ok(())
}
