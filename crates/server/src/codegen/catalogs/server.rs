#![allow(unused_mut)]
use super::handler::CatalogHandler;
use crate::Result;
use crate::api::RequestContext;
use crate::policy::Recipient;
use axum::extract::{Extension, State};
use unitycatalog_common::models::catalogs::v1::*;
pub async fn list_catalogs<T: CatalogHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: ListCatalogsRequest,
) -> Result<::axum::Json<ListCatalogsResponse>> {
    let context = RequestContext { recipient };
    let result = handler.list_catalogs(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_catalog<T: CatalogHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: CreateCatalogRequest,
) -> Result<::axum::Json<CatalogInfo>> {
    let context = RequestContext { recipient };
    let result = handler.create_catalog(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_catalog<T: CatalogHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: GetCatalogRequest,
) -> Result<::axum::Json<CatalogInfo>> {
    let context = RequestContext { recipient };
    let result = handler.get_catalog(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_catalog<T: CatalogHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: UpdateCatalogRequest,
) -> Result<::axum::Json<CatalogInfo>> {
    let context = RequestContext { recipient };
    let result = handler.update_catalog(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_catalog<T: CatalogHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: DeleteCatalogRequest,
) -> Result<()> {
    let context = RequestContext { recipient };
    handler.delete_catalog(request, context).await?;
    Ok(())
}
