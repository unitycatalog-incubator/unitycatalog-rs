use super::handler::ShareHandler;
use crate::api::RequestContext;
use crate::models::shares::v1::*;
use crate::services::Recipient;
use crate::Result;
pub async fn list_shares_handler<T: ShareHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: ListSharesRequest,
) -> Result<::axum::Json<ListSharesResponse>> {
    let context = RequestContext { recipient };
    let result = handler.list_shares(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_share_handler<T: ShareHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: CreateShareRequest,
) -> Result<::axum::Json<ShareInfo>> {
    let context = RequestContext { recipient };
    let result = handler.create_share(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_share_handler<T: ShareHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: GetShareRequest,
) -> Result<::axum::Json<ShareInfo>> {
    let context = RequestContext { recipient };
    let result = handler.get_share(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_share_handler<T: ShareHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: UpdateShareRequest,
) -> Result<::axum::Json<ShareInfo>> {
    let context = RequestContext { recipient };
    let result = handler.update_share(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_share_handler<T: ShareHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: DeleteShareRequest,
) -> Result<()> {
    let context = RequestContext { recipient };
    handler.delete_share(request, context).await?;
    Ok(())
}
