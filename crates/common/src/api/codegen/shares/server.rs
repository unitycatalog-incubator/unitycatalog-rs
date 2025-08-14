#![allow(unused_mut)]
use super::handler::ShareHandler;
use crate::Result;
use crate::api::RequestContext;
use crate::models::shares::v1::*;
use crate::services::Recipient;
use axum::extract::{Extension, State};
use axum::{RequestExt, RequestPartsExt};
pub async fn list_shares_handler<T: ShareHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: ListSharesRequest,
) -> Result<::axum::Json<ListSharesResponse>> {
    let context = RequestContext { recipient };
    let result = handler.list_shares(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_share_handler<T: ShareHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: CreateShareRequest,
) -> Result<::axum::Json<ShareInfo>> {
    let context = RequestContext { recipient };
    let result = handler.create_share(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_share_handler<T: ShareHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: GetShareRequest,
) -> Result<::axum::Json<ShareInfo>> {
    let context = RequestContext { recipient };
    let result = handler.get_share(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_share_handler<T: ShareHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: UpdateShareRequest,
) -> Result<::axum::Json<ShareInfo>> {
    let context = RequestContext { recipient };
    let result = handler.update_share(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_share_handler<T: ShareHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: DeleteShareRequest,
) -> Result<()> {
    let context = RequestContext { recipient };
    handler.delete_share(request, context).await?;
    Ok(())
}
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for ListSharesRequest {
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
        Ok(ListSharesRequest {
            max_results,
            page_token,
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequest<S> for CreateShareRequest {
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
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for GetShareRequest {
    type Rejection = crate::Error;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path(name) = parts.extract::<axum::extract::Path<String>>().await?;
        #[derive(serde::Deserialize)]
        struct QueryParams {
            #[serde(default)]
            include_shared_data: Option<bool>,
        }
        let axum::extract::Query(QueryParams {
            include_shared_data,
        }) = parts.extract::<axum::extract::Query<QueryParams>>().await?;
        Ok(GetShareRequest {
            name,
            include_shared_data,
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequest<S> for UpdateShareRequest {
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
        let axum::extract::Json::<UpdateShareRequest>(body) = body_req
            .extract()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        let (updates, new_name, owner, comment) =
            (body.updates, body.new_name, body.owner, body.comment);
        Ok(UpdateShareRequest {
            name,
            updates,
            new_name,
            owner,
            comment,
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for DeleteShareRequest {
    type Rejection = crate::Error;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path(name) = parts.extract::<axum::extract::Path<String>>().await?;
        Ok(DeleteShareRequest { name })
    }
}
