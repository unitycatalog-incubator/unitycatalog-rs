#![allow(unused_mut)]
use crate::Result;
use crate::models::sharing::v1::*;
use axum::{RequestExt, RequestPartsExt};

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

impl<S: Send + Sync> axum::extract::FromRequestParts<S> for GetShareRequest {
    type Rejection = crate::Error;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path(name) = parts.extract::<axum::extract::Path<String>>().await?;
        Ok(GetShareRequest { name })
    }
}

impl<S: Send + Sync> axum::extract::FromRequestParts<S> for ListSharingSchemasRequest {
    type Rejection = crate::Error;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path(share) = parts.extract::<axum::extract::Path<String>>().await?;
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
        Ok(ListSharingSchemasRequest {
            share,
            max_results,
            page_token,
        })
    }
}

impl<S: Send + Sync> axum::extract::FromRequestParts<S> for ListSchemaTablesRequest {
    type Rejection = crate::Error;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path((share, name)) = parts
            .extract::<axum::extract::Path<(String, String)>>()
            .await?;
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
        Ok(ListSchemaTablesRequest {
            share,
            name,
            max_results,
            page_token,
        })
    }
}

impl<S: Send + Sync> axum::extract::FromRequestParts<S> for ListShareTablesRequest {
    type Rejection = crate::Error;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path(name) = parts.extract::<axum::extract::Path<String>>().await?;
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
        Ok(ListShareTablesRequest {
            name,
            max_results,
            page_token,
        })
    }
}

impl<S: Send + Sync> axum::extract::FromRequestParts<S> for GetTableVersionRequest {
    type Rejection = crate::Error;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path((share, schema, name)) = parts
            .extract::<axum::extract::Path<(String, String, String)>>()
            .await?;
        #[derive(serde::Deserialize)]
        struct QueryParams {
            #[serde(default)]
            starting_timestamp: Option<String>,
        }
        let axum::extract::Query(QueryParams { starting_timestamp }) =
            parts.extract::<axum::extract::Query<QueryParams>>().await?;
        Ok(GetTableVersionRequest {
            share,
            schema,
            name,
            starting_timestamp,
        })
    }
}

impl<S: Send + Sync> axum::extract::FromRequestParts<S> for GetTableMetadataRequest {
    type Rejection = crate::Error;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path((share, schema, name)) = parts
            .extract::<axum::extract::Path<(String, String, String)>>()
            .await?;
        Ok(GetTableMetadataRequest {
            share,
            schema,
            name,
        })
    }
}

impl<S: Send + Sync> axum::extract::FromRequest<S> for QueryTableRequest {
    type Rejection = axum::response::Response;
    async fn from_request(
        mut req: axum::extract::Request<axum::body::Body>,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = req.into_parts();
        let axum::extract::Path((share, schema, name)) = parts
            .extract::<axum::extract::Path<(String, String, String)>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        let body_req = axum::extract::Request::from_parts(parts, body);
        let axum::extract::Json::<QueryTableRequest>(body) = body_req
            .extract()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        let (
            starting_timestamp,
            predicate_hints,
            json_predicate_hints,
            limit_hint,
            version,
            timestamp,
            starting_version,
            ending_version,
        ) = (
            body.starting_timestamp,
            body.predicate_hints,
            body.json_predicate_hints,
            body.limit_hint,
            body.version,
            body.timestamp,
            body.starting_version,
            body.ending_version,
        );
        Ok(QueryTableRequest {
            share,
            schema,
            name,
            starting_timestamp,
            predicate_hints,
            json_predicate_hints,
            limit_hint,
            version,
            timestamp,
            starting_version,
            ending_version,
        })
    }
}
