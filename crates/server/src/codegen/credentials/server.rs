#![allow(unused_mut)]
use unitycatalog_common::Result;
use crate::api::RequestContext;
use unitycatalog_common::models::credentials::v1::*;
use super::handler::CredentialHandler;
use crate::policy::Recipient;
use axum::extract::{State, Extension};
pub async fn list_credentials_handler<T: CredentialHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: ListCredentialsRequest,
) -> Result<::axum::Json<ListCredentialsResponse>> {
    let context = RequestContext { recipient };
    let result = handler.list_credentials(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_credential_handler<T: CredentialHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: CreateCredentialRequest,
) -> Result<::axum::Json<CredentialInfo>> {
    let context = RequestContext { recipient };
    let result = handler.create_credential(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_credential_handler<T: CredentialHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: GetCredentialRequest,
) -> Result<::axum::Json<CredentialInfo>> {
    let context = RequestContext { recipient };
    let result = handler.get_credential(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_credential_handler<T: CredentialHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: UpdateCredentialRequest,
) -> Result<::axum::Json<CredentialInfo>> {
    let context = RequestContext { recipient };
    let result = handler.update_credential(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_credential_handler<T: CredentialHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: DeleteCredentialRequest,
) -> Result<()> {
    let context = RequestContext { recipient };
    handler.delete_credential(request, context).await?;
    Ok(())
}
