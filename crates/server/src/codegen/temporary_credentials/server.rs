#![allow(unused_mut)]
use super::handler::TemporaryCredentialHandler;
use crate::Result;
use crate::api::RequestContext;
use crate::policy::Recipient;
use axum::extract::{Extension, State};
use unitycatalog_common::models::temporary_credentials::v1::*;
pub async fn generate_temporary_table_credentials<T: TemporaryCredentialHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: GenerateTemporaryTableCredentialsRequest,
) -> Result<::axum::Json<TemporaryCredential>> {
    let context = RequestContext { recipient };
    let result = handler
        .generate_temporary_table_credentials(request, context)
        .await?;
    Ok(axum::Json(result))
}
pub async fn generate_temporary_path_credentials<T: TemporaryCredentialHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: GenerateTemporaryPathCredentialsRequest,
) -> Result<::axum::Json<TemporaryCredential>> {
    let context = RequestContext { recipient };
    let result = handler
        .generate_temporary_path_credentials(request, context)
        .await?;
    Ok(axum::Json(result))
}
