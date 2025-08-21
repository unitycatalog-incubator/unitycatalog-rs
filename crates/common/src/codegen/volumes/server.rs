#![allow(unused_mut)]
use crate::Result;
use crate::models::volumes::v1::*;
use axum::{RequestExt, RequestPartsExt};
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for ListVolumesRequest {
    type Rejection = crate::Error;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        #[derive(serde::Deserialize)]
        struct QueryParams {
            catalog_name: String,
            schema_name: String,
            #[serde(default)]
            max_results: Option<i32>,
            #[serde(default)]
            page_token: Option<String>,
            #[serde(default)]
            include_browse: Option<bool>,
        }
        let axum::extract::Query(QueryParams {
            catalog_name,
            schema_name,
            max_results,
            page_token,
            include_browse,
        }) = parts.extract::<axum::extract::Query<QueryParams>>().await?;
        Ok(ListVolumesRequest {
            catalog_name,
            schema_name,
            max_results,
            page_token,
            include_browse,
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequest<S> for CreateVolumeRequest {
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
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for GetVolumeRequest {
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
        Ok(GetVolumeRequest {
            name,
            include_browse,
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequest<S> for UpdateVolumeRequest {
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
        let axum::extract::Json::<UpdateVolumeRequest>(body) = body_req
            .extract()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        let (new_name, comment, owner, include_browse) =
            (body.new_name, body.comment, body.owner, body.include_browse);
        Ok(UpdateVolumeRequest {
            name,
            new_name,
            comment,
            owner,
            include_browse,
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for DeleteVolumeRequest {
    type Rejection = crate::Error;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path(name) = parts.extract::<axum::extract::Path<String>>().await?;
        Ok(DeleteVolumeRequest { name })
    }
}
