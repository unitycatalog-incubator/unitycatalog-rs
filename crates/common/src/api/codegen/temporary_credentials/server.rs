#![allow(unused_mut)]
use super::handler::TemporaryCredentialHandler;
use crate::Result;
use crate::api::RequestContext;
use crate::models::temporary_credentials::v1::*;
use crate::services::Recipient;
use axum::extract::{Extension, State};
use axum::{RequestExt, RequestPartsExt};
pub async fn generate_temporary_table_credentials_handler<T: TemporaryCredentialHandler>(
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
pub async fn generate_temporary_volume_credentials_handler<T: TemporaryCredentialHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: GenerateTemporaryVolumeCredentialsRequest,
) -> Result<::axum::Json<TemporaryCredential>> {
    let context = RequestContext { recipient };
    let result = handler
        .generate_temporary_volume_credentials(request, context)
        .await?;
    Ok(axum::Json(result))
}
impl<S: Send + Sync> axum::extract::FromRequest<S> for GenerateTemporaryTableCredentialsRequest {
    type Rejection = axum::response::Response;
    async fn from_request(
        req: axum::extract::Request<axum::body::Body>,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Json(request) = req
            .extract()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(request)
    }
}
impl<S: Send + Sync> axum::extract::FromRequest<S> for GenerateTemporaryVolumeCredentialsRequest {
    type Rejection = axum::response::Response;
    async fn from_request(
        req: axum::extract::Request<axum::body::Body>,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Json(request) = req
            .extract()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(request)
    }
}
