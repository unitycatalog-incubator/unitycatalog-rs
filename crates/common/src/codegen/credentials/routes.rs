use super::handler::CredentialHandler;
use crate::api::RequestContext;
use crate::models::credentials::v1::*;
use crate::services::Recipient;
use crate::Result;
pub async fn list_credentials_handler<T: CredentialHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: ListCredentialsRequest,
) -> Result<::axum::Json<ListCredentialsResponse>> {
    let context = RequestContext { recipient };
    let result = handler.list_credentials(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_credential_handler<T: CredentialHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: CreateCredentialRequest,
) -> Result<::axum::Json<CredentialInfo>> {
    let context = RequestContext { recipient };
    let result = handler.create_credential(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_credential_handler<T: CredentialHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: GetCredentialRequest,
) -> Result<::axum::Json<CredentialInfo>> {
    let context = RequestContext { recipient };
    let result = handler.get_credential(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_credential_handler<T: CredentialHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: UpdateCredentialRequest,
) -> Result<::axum::Json<CredentialInfo>> {
    let context = RequestContext { recipient };
    let result = handler.update_credential(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_credential_handler<T: CredentialHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: DeleteCredentialRequest,
) -> Result<()> {
    let context = RequestContext { recipient };
    handler.delete_credential(request, context).await?;
    Ok(())
}
