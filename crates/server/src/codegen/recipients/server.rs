#![allow(unused_mut)]
use super::handler::RecipientHandler;
use crate::Result;
use crate::api::RequestContext;
use crate::policy::Principal;
use axum::extract::{Extension, State};
use unitycatalog_common::models::recipients::v1::*;
pub async fn list_recipients<T: RecipientHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Principal>,
    request: ListRecipientsRequest,
) -> Result<::axum::Json<ListRecipientsResponse>> {
    let context = RequestContext { recipient };
    let result = handler.list_recipients(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_recipient<T: RecipientHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Principal>,
    request: CreateRecipientRequest,
) -> Result<::axum::Json<Recipient>> {
    let context = RequestContext { recipient };
    let result = handler.create_recipient(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_recipient<T: RecipientHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Principal>,
    request: GetRecipientRequest,
) -> Result<::axum::Json<Recipient>> {
    let context = RequestContext { recipient };
    let result = handler.get_recipient(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_recipient<T: RecipientHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Principal>,
    request: UpdateRecipientRequest,
) -> Result<::axum::Json<Recipient>> {
    let context = RequestContext { recipient };
    let result = handler.update_recipient(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_recipient<T: RecipientHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Principal>,
    request: DeleteRecipientRequest,
) -> Result<()> {
    let context = RequestContext { recipient };
    handler.delete_recipient(request, context).await?;
    Ok(())
}
