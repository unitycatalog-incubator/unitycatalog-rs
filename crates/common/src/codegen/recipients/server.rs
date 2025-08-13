#![allow(unused_mut)]
use super::handler::RecipientHandler;
use crate::Result;
use crate::api::RequestContext;
use crate::models::recipients::v1::*;
use crate::services::Recipient;
use axum::extract::{Extension, State};
use axum::{RequestExt, RequestPartsExt};
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
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for ListRecipientsRequest {
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
        }
        let axum::extract::Query(QueryParams {
            max_results,
            page_token,
        }) = parts.extract::<axum::extract::Query<QueryParams>>().await?;
        Ok(ListRecipientsRequest {
            max_results,
            page_token,
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequest<S> for CreateRecipientRequest {
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
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for GetRecipientRequest {
    type Rejection = crate::Error;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path((name)) = parts.extract::<axum::extract::Path<(String)>>().await?;
        Ok(GetRecipientRequest { name })
    }
}
impl<S: Send + Sync> axum::extract::FromRequest<S> for UpdateRecipientRequest {
    type Rejection = axum::response::Response;
    async fn from_request(
        mut req: axum::extract::Request<axum::body::Body>,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = req.into_parts();
        let axum::extract::Path((name)) = parts
            .extract::<axum::extract::Path<(String)>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        let body_req = axum::extract::Request::from_parts(parts, body);
        let axum::extract::Json::<UpdateRecipientRequest>(body) = body_req
            .extract()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        let (new_name, owner, comment, properties, expiration_time) = (
            body.new_name,
            body.owner,
            body.comment,
            body.properties,
            body.expiration_time,
        );
        Ok(UpdateRecipientRequest {
            name,
            new_name,
            owner,
            comment,
            properties,
            expiration_time,
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for DeleteRecipientRequest {
    type Rejection = crate::Error;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path((name)) = parts.extract::<axum::extract::Path<(String)>>().await?;
        Ok(DeleteRecipientRequest { name })
    }
}
