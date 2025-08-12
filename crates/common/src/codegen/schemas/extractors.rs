use crate::models::schemas::v1::*;
use ::axum::{RequestExt, RequestPartsExt};
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for ListSchemasRequest {
    type Rejection = crate::Error;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        #[derive(serde::Deserialize)]
        struct QueryParams {
            catalog_name: String,
            #[serde(default)]
            max_results: Option<i32>,
            #[serde(default)]
            page_token: Option<String>,
            #[serde(default)]
            include_browse: Option<bool>,
        }
        let axum::extract::Query(QueryParams {
            catalog_name,
            max_results,
            page_token,
            include_browse,
        }) = parts.extract::<axum::extract::Query<QueryParams>>().await?;
        Ok(ListSchemasRequest {
            catalog_name,
            max_results,
            page_token,
            include_browse,
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequest<S> for CreateSchemaRequest {
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
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for GetSchemaRequest {
    type Rejection = crate::Error;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path((full_name)) =
            parts.extract::<axum::extract::Path<(String)>>().await?;
        Ok(GetSchemaRequest { full_name })
    }
}
impl<S: Send + Sync> axum::extract::FromRequest<S> for UpdateSchemaRequest {
    type Rejection = axum::response::Response;
    async fn from_request(
        mut req: axum::extract::Request<axum::body::Body>,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = req.into_parts();
        let axum::extract::Path((full_name)) = parts
            .extract::<axum::extract::Path<(String)>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        let body_req = axum::extract::Request::from_parts(parts, body);
        let axum::extract::Json::<UpdateSchemaRequest>(body) = body_req
            .extract()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        let (comment, properties, new_name) = (body.comment, body.properties, body.new_name);
        Ok(UpdateSchemaRequest {
            full_name,
            comment,
            properties,
            new_name,
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for DeleteSchemaRequest {
    type Rejection = crate::Error;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path((full_name)) =
            parts.extract::<axum::extract::Path<(String)>>().await?;
        #[derive(serde::Deserialize)]
        struct QueryParams {
            #[serde(default)]
            force: Option<bool>,
        }
        let axum::extract::Query(QueryParams { force }) =
            parts.extract::<axum::extract::Query<QueryParams>>().await?;
        Ok(DeleteSchemaRequest { full_name, force })
    }
}
