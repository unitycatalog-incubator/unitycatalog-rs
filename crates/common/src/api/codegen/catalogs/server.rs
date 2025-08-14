#![allow(unused_mut)]
use super::handler::CatalogHandler;
use crate::Result;
use crate::api::RequestContext;
use crate::models::catalogs::v1::*;
use crate::services::Recipient;
use axum::extract::{Extension, State};
use axum::{RequestExt, RequestPartsExt};
pub async fn list_catalogs_handler<T: CatalogHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: ListCatalogsRequest,
) -> Result<::axum::Json<ListCatalogsResponse>> {
    let context = RequestContext { recipient };
    let result = handler.list_catalogs(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_catalog_handler<T: CatalogHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: CreateCatalogRequest,
) -> Result<::axum::Json<CatalogInfo>> {
    let context = RequestContext { recipient };
    let result = handler.create_catalog(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_catalog_handler<T: CatalogHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: GetCatalogRequest,
) -> Result<::axum::Json<CatalogInfo>> {
    let context = RequestContext { recipient };
    let result = handler.get_catalog(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_catalog_handler<T: CatalogHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: UpdateCatalogRequest,
) -> Result<::axum::Json<CatalogInfo>> {
    let context = RequestContext { recipient };
    let result = handler.update_catalog(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_catalog_handler<T: CatalogHandler>(
    State(handler): State<T>,
    Extension(recipient): Extension<Recipient>,
    request: DeleteCatalogRequest,
) -> Result<()> {
    let context = RequestContext { recipient };
    handler.delete_catalog(request, context).await?;
    Ok(())
}
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for ListCatalogsRequest {
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
        Ok(ListCatalogsRequest {
            max_results,
            page_token,
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequest<S> for CreateCatalogRequest {
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
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for GetCatalogRequest {
    type Rejection = crate::Error;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path(name) = parts.extract::<axum::extract::Path<String>>().await?;
        #[derive(serde::Deserialize)]
        struct QueryParams {
            #[serde(default)]
            include_browse: Option<bool>,
        }
        let axum::extract::Query(QueryParams { include_browse }) =
            parts.extract::<axum::extract::Query<QueryParams>>().await?;
        Ok(GetCatalogRequest {
            name,
            include_browse,
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequest<S> for UpdateCatalogRequest {
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
        let axum::extract::Json::<UpdateCatalogRequest>(body) = body_req
            .extract()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        let (owner, comment, properties, new_name) =
            (body.owner, body.comment, body.properties, body.new_name);
        Ok(UpdateCatalogRequest {
            name,
            owner,
            comment,
            properties,
            new_name,
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for DeleteCatalogRequest {
    type Rejection = crate::Error;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path(name) = parts.extract::<axum::extract::Path<String>>().await?;
        #[derive(serde::Deserialize)]
        struct QueryParams {
            #[serde(default)]
            force: Option<bool>,
        }
        let axum::extract::Query(QueryParams { force }) =
            parts.extract::<axum::extract::Query<QueryParams>>().await?;
        Ok(DeleteCatalogRequest { name, force })
    }
}
