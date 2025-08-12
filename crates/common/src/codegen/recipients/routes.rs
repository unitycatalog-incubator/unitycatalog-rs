use super::handler::RecipientHandler;
use crate::api::RequestContext;
use crate::models::recipients::v1::*;
use crate::services::Recipient;
use crate::Result;
pub async fn list_recipients_handler<T: RecipientHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: ListRecipientsRequest,
) -> Result<::axum::Json<ListRecipientsResponse>> {
    let context = RequestContext { recipient };
    let result = handler.list_recipients(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_recipient_handler<T: RecipientHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: CreateRecipientRequest,
) -> Result<::axum::Json<RecipientInfo>> {
    let context = RequestContext { recipient };
    let result = handler.create_recipient(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_recipient_handler<T: RecipientHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: GetRecipientRequest,
) -> Result<::axum::Json<RecipientInfo>> {
    let context = RequestContext { recipient };
    let result = handler.get_recipient(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_recipient_handler<T: RecipientHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: UpdateRecipientRequest,
) -> Result<::axum::Json<RecipientInfo>> {
    let context = RequestContext { recipient };
    let result = handler.update_recipient(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_recipient_handler<T: RecipientHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: DeleteRecipientRequest,
) -> Result<()> {
    let context = RequestContext { recipient };
    handler.delete_recipient(request, context).await?;
    Ok(())
}
