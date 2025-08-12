use super::handler::TemporaryCredentialHandler;
use crate::api::RequestContext;
use crate::models::temporary_credentials::v1::*;
use crate::services::Recipient;
use crate::Result;
pub async fn generate_temporary_table_credentials_handler<T: TemporaryCredentialHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: GenerateTemporaryTableCredentialsRequest,
) -> Result<::axum::Json<TemporaryCredential>> {
    let context = RequestContext { recipient };
    let result = handler
        .generate_temporary_table_credentials(request, context)
        .await?;
    Ok(axum::Json(result))
}
pub async fn generate_temporary_volume_credentials_handler<T: TemporaryCredentialHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: GenerateTemporaryVolumeCredentialsRequest,
) -> Result<::axum::Json<TemporaryCredential>> {
    let context = RequestContext { recipient };
    let result = handler
        .generate_temporary_volume_credentials(request, context)
        .await?;
    Ok(axum::Json(result))
}
