use bytes::Bytes;
use itertools::Itertools;

use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::models::shares::v1::{
    DataObjectType, GetShareRequest as SharesGetShareRequest, ShareInfo,
};
use unitycatalog_sharing_client::models::sharing::v1::{Share as SharingShare, *};

use crate::Result;
use crate::api::{RequestContext, SecuredAction, ShareHandler};
use crate::policy::{Permission, Policy, process_resources};
pub use crate::sharing::SharingHandler;
use crate::store::ResourceStore;

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
impl<T: ResourceStore + Policy + ShareHandler> SharingHandler for T {
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
            return SharingHandler::list_shares(self, request, context).await;
        }

        let shares: Vec<ShareInfo> = resources.into_iter().map(|r| r.try_into()).try_collect()?;

        Ok(ListSharesResponse {
            items: shares
                .into_iter()
                .map(|r| SharingShare {
                    name: r.name,
                    id: r.id,
                })
                .collect(),
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
        request: ListSchemasRequest,
        context: RequestContext,
    ) -> Result<ListSchemasResponse> {
        self.check_required(&request, context.recipient()).await?;
        let shares_request = SharesGetShareRequest {
            name: request.share,
            include_shared_data: Some(true),
        };
        let share: ShareInfo = self.get(&shares_request.resource()).await?.0.try_into()?;
        Ok(ListSchemasResponse {
            items: share
                .data_objects
                .into_iter()
                .filter_map(|a| {
                    if matches!(a.data_object_type(), DataObjectType::Table) {
                        Some(Schema {
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

    async fn list_tables(
        &self,
        request: ListTablesRequest,
        context: RequestContext,
    ) -> Result<ListTablesResponse> {
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
                        Some(Table {
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
        Ok(ListTablesResponse {
            items,
            next_page_token: None,
        })
    }

    async fn list_all_tables(
        &self,
        request: ListAllTablesRequest,
        context: RequestContext,
    ) -> Result<ListAllTablesResponse> {
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
                    Some(Table {
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
        Ok(ListAllTablesResponse {
            items,
            next_page_token: None,
        })
    }

    async fn get_table_version(
        &self,
        _request: GetTableVersionRequest,
        _context: RequestContext,
    ) -> Result<GetTableVersionResponse> {
        unimplemented!("only method on SharingQueryHandler should be used")
    }
    async fn get_table_metadata(
        &self,
        _request: GetTableMetadataRequest,
        _context: RequestContext,
    ) -> Result<QueryResponse> {
        unimplemented!("only method on SharingQueryHandler should be used")
    }
    async fn query_table(
        &self,
        _request: QueryTableRequest,
        _context: RequestContext,
    ) -> Result<QueryResponse> {
        unimplemented!("only method on SharingQueryHandler should be used")
    }
}
