use bytes::Bytes;

use super::{Policy, ServerHandler, StorageLocationUrl, TableManager};
use crate::api::RequestContext;
use crate::models::sharing::v1::*;
use crate::models::tables::v1::{DataSourceFormat, TableInfo};
use crate::resources::ResourceStore;
use crate::sharing::{
    MetadataResponse, MetadataResponseData, ProtocolResponseData, SharingQueryHandler,
};
use crate::{ResourceIdent, ResourceName, Result, ShareInfo};

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

#[async_trait::async_trait]
pub(super) trait SharingExt: Send + Sync + 'static {
    async fn table_location(&self, table_ref: &SharingTableReference)
    -> Result<StorageLocationUrl>;
}

#[async_trait::async_trait]
impl<T: ResourceStore> SharingExt for T {
    async fn table_location(
        &self,
        table_ref: &SharingTableReference,
    ) -> Result<StorageLocationUrl> {
        let share_ident = ResourceIdent::share(ResourceName::new([table_ref.share.as_str()]));
        let share_info: ShareInfo = self.get(&share_ident).await?.0.try_into()?;
        let Some(table_object) = share_info
            .data_objects
            .iter()
            .find(|o| o.shared_as() == &format!("{}.{}", table_ref.schema, table_ref.table))
        else {
            return Err(crate::Error::NotFound);
        };
        let table_ident = ResourceIdent::table(ResourceName::new(table_object.name.split(".")));
        let table_info: TableInfo = self.get(&table_ident).await?.0.try_into()?;
        let location = table_info.storage_location.ok_or(crate::Error::NotFound)?;
        StorageLocationUrl::parse(&location)
    }
}

#[async_trait::async_trait]
impl SharingQueryHandler for ServerHandler {
    async fn get_table_version(
        &self,
        request: GetTableVersionRequest,
        context: RequestContext,
    ) -> Result<GetTableVersionResponse> {
        self.check_required(&request, context.recipient()).await?;
        let table_ref = SharingTableReference {
            share: request.share,
            schema: request.schema,
            table: request.name,
        };
        let location = self.table_location(&table_ref).await?;
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
        self.check_required(&request, context.recipient()).await?;
        let table_ref = SharingTableReference {
            share: request.share,
            schema: request.schema,
            table: request.name,
        };
        let location = self.table_location(&table_ref).await?;
        let snapshot = self
            .read_snapshot(&location, &DataSourceFormat::Delta, None)
            .await?;

        let mut response = serde_json::to_vec(&MetadataResponse::MetaData(
            MetadataResponseData::ParquetMetadata(snapshot.metadata().try_into()?),
        ))?;
        response.push(b'\n');
        response.extend(serde_json::to_vec(&MetadataResponse::Protocol(
            ProtocolResponseData::ParquetProtocol(snapshot.protocol().into()),
        ))?);

        Ok(Bytes::from(response))
    }

    async fn query_table(
        &self,
        request: QueryTableRequest,
        context: RequestContext,
    ) -> Result<Bytes> {
        self.check_required(&request, context.recipient()).await?;
        let table_ref = SharingTableReference {
            share: request.share,
            schema: request.schema,
            table: request.name,
        };
        let data = self
            .session
            .extract_sharing_query_response(&table_ref, &self.handler)
            .await?;
        Ok(data)
    }
}
