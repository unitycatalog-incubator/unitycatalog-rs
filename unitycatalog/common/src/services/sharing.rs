use super::{Policy, ServerHandler, StorageLocationUrl, TableManager};
use crate::api::{RequestContext, SharingQueryHandler};
use crate::models::sharing::v1::*;
use crate::models::tables::v1::{DataSourceFormat, TableInfo};
use crate::resources::ResourceStore;
use crate::{ResourceIdent, ResourceName, Result, ShareInfo};

#[async_trait::async_trait]
trait SharingExt {
    async fn get_snapshot(
        &self,
        share: &str,
        schema: &str,
        table: &str,
    ) -> Result<StorageLocationUrl>;
}

#[async_trait::async_trait]
impl<T: TableManager + ResourceStore> SharingExt for T {
    async fn get_snapshot(
        &self,
        share: &str,
        schema: &str,
        table: &str,
    ) -> Result<StorageLocationUrl> {
        let share_ident = ResourceIdent::share(ResourceName::new([share]));
        let share_info: ShareInfo = self.get(&share_ident).await?.0.try_into()?;
        let Some(table_object) = share_info
            .data_objects
            .iter()
            .find(|o| o.shared_as() == &format!("{}.{}", schema, table))
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
        let location = self
            .get_snapshot(&request.share, &request.schema, &request.name)
            .await?;
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
    ) -> Result<QueryResponse> {
        self.check_required(&request, context.recipient()).await?;
        let location = self
            .get_snapshot(&request.share, &request.schema, &request.name)
            .await?;
        let snapshot = self
            .read_snapshot(&location, &DataSourceFormat::Delta, None)
            .await?;
        Ok([snapshot.metadata().into(), snapshot.protocol().into()].into())
    }

    async fn query_table(
        &self,
        request: QueryTableRequest,
        context: RequestContext,
    ) -> Result<QueryResponse> {
        self.check_required(&request, context.recipient()).await?;
        todo!()
    }
}
