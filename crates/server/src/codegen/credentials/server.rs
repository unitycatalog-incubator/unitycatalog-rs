#![allow(unused_mut)]
use super::handler::CredentialHandler;
use crate::Result;
use crate::api::RequestContext;
use crate::policy::Principal;
use axum::extract::{Extension, State};
use unitycatalog_common::models::credentials::v1::*;
pub async fn list_credentials<T: CredentialHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Principal>,
    request: ListCredentialsRequest,
) -> Result<::axum::Json<ListCredentialsResponse>> {
    let context = RequestContext { recipient };
    let result = handler.list_credentials(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_credential<T: CredentialHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Principal>,
    request: CreateCredentialRequest,
) -> Result<::axum::Json<CredentialInfo>> {
    let context = RequestContext { recipient };
    let result = handler.create_credential(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_credential<T: CredentialHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Principal>,
    request: GetCredentialRequest,
) -> Result<::axum::Json<CredentialInfo>> {
    let context = RequestContext { recipient };
    let result = handler.get_credential(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_credential<T: CredentialHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Principal>,
    request: UpdateCredentialRequest,
) -> Result<::axum::Json<CredentialInfo>> {
    let context = RequestContext { recipient };
    let result = handler.update_credential(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_credential<T: CredentialHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Principal>,
    request: DeleteCredentialRequest,
) -> Result<()> {
    let context = RequestContext { recipient };
    handler.delete_credential(request, context).await?;
    Ok(())
}
