use bytes::Bytes;

use unitycatalog_common::models::tables::v1::{DataSourceFormat, GetTableRequest, Table};
use unitycatalog_common::{ResourceIdent, ResourceName, Share};
use unitycatalog_sharing_client::models::sharing::v1::*;

use super::{Policy, ServerHandler, StorageLocationUrl, TableManager};
use crate::api::RequestContext;
use crate::api::sharing::{
    MetadataResponse, MetadataResponseData, ProtocolResponseData, SharingQueryHandler,
};
use crate::error::{Error, Result};
use crate::store::ResourceStoreReader;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharingTableReference {
    share: String,
    schema: String,
    table: String,
}

impl SharingTableReference {
    pub(super) fn system_table_name(&self) -> String {
        format!("{}__{}__{}", self.share, self.schema, self.table)
    }
}

impl ServerHandler<RequestContext> {
    /// Resolve the storage location of a shared table.
    ///
    /// The Share itself is always read from the local store (shares are a
    /// sharing-server-owned primitive). The backing Table primitive is resolved
    /// through the configured [`table_source`](ServerHandler::table_source) when
    /// present — so in the side-by-side topology it is fetched from the upstream
    /// Unity Catalog rather than the local store — and falls back to a local
    /// store lookup otherwise.
    pub(super) async fn resolve_table_location(
        &self,
        table_ref: &SharingTableReference,
        context: &RequestContext,
    ) -> Result<StorageLocationUrl> {
        let share_ident = ResourceIdent::share(ResourceName::new([table_ref.share.as_str()]));
        let share_info: Share = self.get(&share_ident).await?.0.try_into()?;
        let Some(table_object) = share_info
            .objects
            .iter()
            .find(|o| o.shared_as() == format!("{}.{}", table_ref.schema, table_ref.table))
        else {
            return Err(Error::NotFound);
        };

        let table_info: Table = if let Some(table_source) = self.table_source() {
            // Side-by-side topology: resolve the Table primitive through the
            // routed handler (e.g. upstream Unity Catalog), keyed by full name.
            let request = GetTableRequest {
                full_name: table_object.name.clone(),
                ..Default::default()
            };
            table_source.get_table(request, context.clone()).await?
        } else {
            // Self-contained topology: the Table primitive lives in the local
            // store alongside the Share.
            let table_ident = ResourceIdent::table(ResourceName::new(table_object.name.split(".")));
            self.get(&table_ident).await?.0.try_into()?
        };

        let location = table_info.storage_location.ok_or(Error::NotFound)?;
        Ok(StorageLocationUrl::parse(&location)?)
    }
}

#[async_trait::async_trait]
impl SharingQueryHandler for ServerHandler<RequestContext> {
    async fn get_table_version(
        &self,
        request: GetTableVersionRequest,
        context: RequestContext,
    ) -> Result<GetTableVersionResponse> {
        self.check_required(&request, &context).await?;
        let table_ref = SharingTableReference {
            share: request.share,
            schema: request.schema,
            table: request.name,
        };
        let location = self.resolve_table_location(&table_ref, &context).await?;
        let snapshot = self
            .read_snapshot(&location, &DataSourceFormat::Delta, None)
            .await?;
        Ok(GetTableVersionResponse {
            version: snapshot.version() as i64,
        })
    }

    async fn get_table_metadata(
        &self,
        request: GetTableMetadataRequest,
        context: RequestContext,
    ) -> Result<Bytes> {
        self.check_required(&request, &context).await?;
        let table_ref = SharingTableReference {
            share: request.share,
            schema: request.schema,
            table: request.name,
        };
        let location = self.resolve_table_location(&table_ref, &context).await?;
        let snapshot = self
            .read_snapshot(&location, &DataSourceFormat::Delta, None)
            .await?;

        let table_config = snapshot.table_configuration();
        let mut response = serde_json::to_vec(&MetadataResponse::MetaData(
            MetadataResponseData::ParquetMetadata(table_config.metadata().try_into()?),
        ))?;
        response.push(b'\n');
        response.extend(serde_json::to_vec(&MetadataResponse::Protocol(
            ProtocolResponseData::ParquetProtocol(table_config.protocol().into()),
        ))?);

        Ok(Bytes::from(response))
    }

    async fn query_table(
        &self,
        request: QueryTableRequest,
        context: RequestContext,
    ) -> Result<Bytes> {
        self.check_required(&request, &context).await?;
        let table_ref = SharingTableReference {
            share: request.share,
            schema: request.schema,
            table: request.name,
        };
        let location = self.resolve_table_location(&table_ref, &context).await?;
        let data = self
            .session
            .extract_sharing_query_response(&table_ref, &location)
            .await?;
        Ok(data)
    }
}
