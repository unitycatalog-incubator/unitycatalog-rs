#![allow(unused_mut)]
use super::handler::CredentialHandler;
use crate::Result;
use crate::api::RequestContext;
use crate::models::credentials::v1::*;
use crate::services::Recipient;
use axum::extract::{Extension, State};
use axum::{RequestExt, RequestPartsExt};
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
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for ListCredentialsRequest {
    type Rejection = crate::Error;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        #[derive(serde::Deserialize)]
        struct QueryParams {
            #[serde(default)]
            max_results: Option<i32>,
            #[serde(default)]
            page_token: Option<String>,
            #[serde(default)]
            purpose: Option<i32>,
        }
        let axum::extract::Query(QueryParams {
            max_results,
            page_token,
            purpose,
        }) = parts.extract::<axum::extract::Query<QueryParams>>().await?;
        Ok(ListCredentialsRequest {
            max_results,
            page_token,
            purpose,
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequest<S> for CreateCredentialRequest {
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
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for GetCredentialRequest {
    type Rejection = crate::Error;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path(name) = parts.extract::<axum::extract::Path<String>>().await?;
        Ok(GetCredentialRequest { name })
    }
}
impl<S: Send + Sync> axum::extract::FromRequest<S> for UpdateCredentialRequest {
    type Rejection = axum::response::Response;
    async fn from_request(
        mut req: axum::extract::Request<axum::body::Body>,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = req.into_parts();
        let axum::extract::Path(name) = parts
            .extract::<axum::extract::Path<String>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        let body_req = axum::extract::Request::from_parts(parts, body);
        let axum::extract::Json::<UpdateCredentialRequest>(body) = body_req
            .extract()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(UpdateCredentialRequest {
            name,
            new_name: body.new_name,
            comment: body.comment,
            read_only: body.read_only,
            owner: body.owner,
            skip_validation: body.skip_validation,
            force: body.force,
            credential: body.credential,
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for DeleteCredentialRequest {
    type Rejection = crate::Error;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path(name) = parts.extract::<axum::extract::Path<String>>().await?;
        Ok(DeleteCredentialRequest { name })
    }
}
