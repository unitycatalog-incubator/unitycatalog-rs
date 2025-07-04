use bytes::Bytes;
use itertools::Itertools;
use unitycatalog_derive::rest_handlers;

use crate::api::{RequestContext, SecuredAction, SharesHandler};
use crate::models::ObjectLabel;
use crate::models::shares::v1::{GetShareRequest as SharesGetShareRequest, ShareInfo};
use crate::models::sharing::v1::*;
use crate::resources::{ResourceIdent, ResourceName, ResourceRef, ResourceStore};
use crate::services::policy::{Permission, Policy, Recipient, process_resources};
use crate::shares::v1::DataObjectType;
use crate::{Error, Result};

rest_handlers!(
    SharingDiscoveryHandler, "shares/schemas/tables", [
        ListSharesRequest, Share, Use, ListSharesResponse;
        GetShareRequest, Share, Use, Share with [
            name: path as String,
        ];
        ListSharingSchemasRequest, Share, Use, ListSharingSchemasResponse with [
            share: path as String,
        ];
        ListShareTablesRequest, Share, Use, ListShareTablesResponse with [
            name: path as String,
        ];
        ListSchemaTablesRequest, SharingSchema, Use, ListSchemaTablesResponse with [
            share: path as String,
            name: path as String,
        ];
    ]
);

#[async_trait::async_trait]
pub trait SharingDiscoveryHandler: Send + Sync + 'static {
    /// List all shares that the recipient is allowed to read.
    async fn list_shares(
        &self,
        request: ListSharesRequest,
        context: RequestContext,
    ) -> Result<ListSharesResponse>;

    /// Get a share by name.
    async fn get_share(&self, request: GetShareRequest, context: RequestContext) -> Result<Share>;

    /// List all schemas in a share.
    async fn list_sharing_schemas(
        &self,
        request: ListSharingSchemasRequest,
        context: RequestContext,
    ) -> Result<ListSharingSchemasResponse>;

    /// List all tables in a schema.
    async fn list_schema_tables(
        &self,
        request: ListSchemaTablesRequest,
        context: RequestContext,
    ) -> Result<ListSchemaTablesResponse>;

    /// List all tables in a share.
    async fn list_share_tables(
        &self,
        request: ListShareTablesRequest,
        context: RequestContext,
    ) -> Result<ListShareTablesResponse>;
}

impl SecuredAction for QueryTableRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::share(ResourceName::new([self.share.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for GetTableVersionRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::share(ResourceName::new([self.share.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for GetTableMetadataRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::share(ResourceName::new([self.share.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

#[cfg(feature = "axum")]
mod sharing_impl {
    use axum::body::Body;
    use axum::extract::{FromRequest, FromRequestParts, Json, Path, Query, Request};
    use axum::http::request::Parts;
    use axum::{RequestExt, RequestPartsExt};

    use super::*;

    impl<S: Send + Sync> FromRequestParts<S> for GetTableVersionRequest {
        type Rejection = Error;

        async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
            let Path((share, schema, name)) =
                parts.extract::<Path<(String, String, String)>>().await?;
            let Query(InternalGetTableVersionParams { starting_timestamp }) = parts
                .extract::<Query<InternalGetTableVersionParams>>()
                .await?;
            Ok(GetTableVersionRequest {
                share,
                schema,
                name,
                starting_timestamp,
            })
        }
    }

    impl<S: Send + Sync> FromRequestParts<S> for GetTableMetadataRequest {
        type Rejection = Error;

        async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
            let Path((share, schema, name)) =
                parts.extract::<Path<(String, String, String)>>().await?;
            Ok(GetTableMetadataRequest {
                share,
                schema,
                name,
            })
        }
    }

    impl<S: Send + Sync> FromRequest<S> for QueryTableRequest {
        type Rejection = axum::response::Response;

        async fn from_request(req: Request<Body>, _: &S) -> Result<Self, Self::Rejection> {
            let Json(request) = req
                .extract()
                .await
                .map_err(::axum::response::IntoResponse::into_response)?;
            Ok(request)
        }
    }
}

#[async_trait::async_trait]
pub trait SharingQueryHandler: Send + Sync + 'static {
    async fn get_table_version(
        &self,
        request: GetTableVersionRequest,
        context: RequestContext,
    ) -> Result<GetTableVersionResponse>;

    async fn get_table_metadata(
        &self,
        request: GetTableMetadataRequest,
        context: RequestContext,
    ) -> Result<Bytes>;

    async fn query_table(
        &self,
        request: QueryTableRequest,
        context: RequestContext,
    ) -> Result<Bytes>;
}

#[async_trait::async_trait]
impl<T: ResourceStore + Policy + SharesHandler> SharingDiscoveryHandler for T {
    async fn list_shares(
        &self,
        request: ListSharesRequest,
        context: RequestContext,
    ) -> Result<ListSharesResponse> {
        self.check_required(&request, context.as_ref()).await?;
        let (mut resources, next_page_token) = self
            .list(
                &ObjectLabel::ShareInfo,
                None,
                request.max_results.map(|v| v as usize),
                request.page_token.clone(),
            )
            .await?;
        process_resources(self, context.as_ref(), &Permission::Read, &mut resources).await?;

        // if all resources gor filtered, but there are more pages, try again
        if resources.is_empty() && next_page_token.is_some() {
            return SharingDiscoveryHandler::list_shares(self, request, context).await;
        }

        Ok(ListSharesResponse {
            items: resources.into_iter().map(|r| r.try_into()).try_collect()?,
            next_page_token,
        })
    }

    async fn get_share(&self, request: GetShareRequest, context: RequestContext) -> Result<Share> {
        self.check_required(&request, context.recipient()).await?;
        let shares_request = SharesGetShareRequest {
            name: request.name,
            include_shared_data: Some(false),
        };
        let share: ShareInfo = self.get(&shares_request.resource()).await?.0.try_into()?;
        Ok(Share {
            name: share.name,
            id: share.id,
        })
    }

    async fn list_sharing_schemas(
        &self,
        request: ListSharingSchemasRequest,
        context: RequestContext,
    ) -> Result<ListSharingSchemasResponse> {
        self.check_required(&request, context.recipient()).await?;
        let shares_request = SharesGetShareRequest {
            name: request.share,
            include_shared_data: Some(true),
        };
        let share: ShareInfo = self.get(&shares_request.resource()).await?.0.try_into()?;
        Ok(ListSharingSchemasResponse {
            items: share
                .data_objects
                .into_iter()
                .filter_map(|a| {
                    if matches!(a.data_object_type(), DataObjectType::Table) {
                        Some(SharingSchema {
                            name: a.shared_as().split_once(".")?.0.to_string(),
                            share: share.name.clone(),
                            ..Default::default()
                        })
                    } else {
                        None
                    }
                })
                .dedup()
                .collect(),
            next_page_token: None,
        })
    }

    async fn list_schema_tables(
        &self,
        request: ListSchemaTablesRequest,
        context: RequestContext,
    ) -> Result<ListSchemaTablesResponse> {
        self.check_required(&request, context.recipient()).await?;
        let shares_request = SharesGetShareRequest {
            name: request.share,
            include_shared_data: Some(true),
        };
        let share: ShareInfo = self.get(&shares_request.resource()).await?.0.try_into()?;
        let items = share
            .data_objects
            .into_iter()
            .filter_map(|a| {
                if matches!(a.data_object_type(), DataObjectType::Table) {
                    let (schema, name) = a.shared_as().split_once(".")?;
                    if schema == request.name {
                        Some(SharingTable {
                            name: name.to_string(),
                            share: share.name.clone(),
                            share_id: share.id.clone(),
                            schema: schema.to_string(),
                            ..Default::default()
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        Ok(ListSchemaTablesResponse {
            items,
            next_page_token: None,
        })
    }

    async fn list_share_tables(
        &self,
        request: ListShareTablesRequest,
        context: RequestContext,
    ) -> Result<ListShareTablesResponse> {
        self.check_required(&request, context.recipient()).await?;
        let shares_request = SharesGetShareRequest {
            name: request.name,
            include_shared_data: Some(true),
        };
        let share: ShareInfo = self.get(&shares_request.resource()).await?.0.try_into()?;
        let items = share
            .data_objects
            .into_iter()
            .filter_map(|a| {
                if matches!(a.data_object_type(), DataObjectType::Table) {
                    let (schema, name) = a.shared_as().split_once(".")?;
                    Some(SharingTable {
                        name: name.to_string(),
                        share: share.name.clone(),
                        share_id: share.id.clone(),
                        schema: schema.to_string(),
                        ..Default::default()
                    })
                } else {
                    None
                }
            })
            .collect();
        Ok(ListShareTablesResponse {
            items,
            next_page_token: None,
        })
    }
}
