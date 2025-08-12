use super::handler::CatalogHandler;
use crate::api::RequestContext;
use crate::models::catalogs::v1::*;
use crate::services::Recipient;
use crate::Result;
pub async fn list_catalogs_handler<T: CatalogHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: ListCatalogsRequest,
) -> Result<::axum::Json<ListCatalogsResponse>> {
    let context = RequestContext { recipient };
    let result = handler.list_catalogs(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_catalog_handler<T: CatalogHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: CreateCatalogRequest,
) -> Result<::axum::Json<CatalogInfo>> {
    let context = RequestContext { recipient };
    let result = handler.create_catalog(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_catalog_handler<T: CatalogHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: GetCatalogRequest,
) -> Result<::axum::Json<CatalogInfo>> {
    let context = RequestContext { recipient };
    let result = handler.get_catalog(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_catalog_handler<T: CatalogHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: UpdateCatalogRequest,
) -> Result<::axum::Json<CatalogInfo>> {
    let context = RequestContext { recipient };
    let result = handler.update_catalog(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_catalog_handler<T: CatalogHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: DeleteCatalogRequest,
) -> Result<()> {
    let context = RequestContext { recipient };
    handler.delete_catalog(request, context).await?;
    Ok(())
}
