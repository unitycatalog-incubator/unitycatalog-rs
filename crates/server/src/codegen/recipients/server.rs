#![allow(unused_mut)]
use unitycatalog_common::Result;
use crate::api::RequestContext;
use unitycatalog_common::models::recipients::v1::*;
use super::handler::RecipientHandler;
use crate::policy::Recipient;
use axum::extract::{State, Extension};
pub async fn list_recipients_handler<T: RecipientHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: ListRecipientsRequest,
) -> Result<::axum::Json<ListRecipientsResponse>> {
    let context = RequestContext { recipient };
    let result = handler.list_recipients(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_recipient_handler<T: RecipientHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: CreateRecipientRequest,
) -> Result<::axum::Json<RecipientInfo>> {
    let context = RequestContext { recipient };
    let result = handler.create_recipient(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_recipient_handler<T: RecipientHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: GetRecipientRequest,
) -> Result<::axum::Json<RecipientInfo>> {
    let context = RequestContext { recipient };
    let result = handler.get_recipient(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_recipient_handler<T: RecipientHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: UpdateRecipientRequest,
) -> Result<::axum::Json<RecipientInfo>> {
    let context = RequestContext { recipient };
    let result = handler.update_recipient(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_recipient_handler<T: RecipientHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: DeleteRecipientRequest,
) -> Result<()> {
    let context = RequestContext { recipient };
    handler.delete_recipient(request, context).await?;
    Ok(())
}
