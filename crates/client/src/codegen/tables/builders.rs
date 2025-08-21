#![allow(unused_mut)]
use super::client::*;
use crate::error::Result;
use futures::future::BoxFuture;
use std::future::IntoFuture;
use unitycatalog_common::models::tables::v1::*;
/// Builder for creating requests
pub struct CreateTableBuilder {
    client: TableClient,
    request: CreateTableRequest,
}
impl CreateTableBuilder {
    /// Create a new builder instance
    pub fn new(
        client: TableClient,
        name: impl Into<String>,
        schema_name: impl Into<String>,
        catalog_name: impl Into<String>,
        table_type: TableType,
        data_source_format: DataSourceFormat,
    ) -> Self {
        let request = CreateTableRequest {
            name: name.into(),
            schema_name: schema_name.into(),
            catalog_name: catalog_name.into(),
            table_type: table_type as i32,
            data_source_format: data_source_format as i32,
            ..Default::default()
        };
        Self { client, request }
    }
    #[doc = concat!("Set ", "columns")]
    pub fn with_columns<I>(mut self, columns: I) -> Self
    where
        I: IntoIterator<Item = ColumnInfo>,
    {
        self.request.columns = columns.into_iter().collect();
        self
    }
    #[doc = concat!("Set ", "storage_location")]
    pub fn with_storage_location(mut self, storage_location: impl Into<String>) -> Self {
        self.request.storage_location = Some(storage_location.into());
        self
    }
    #[doc = concat!("Set ", "comment")]
    pub fn with_comment(mut self, comment: impl Into<String>) -> Self {
        self.request.comment = Some(comment.into());
        self
    }
    #[doc = concat!("Set ", "properties")]
    pub fn with_properties<I, K, V>(mut self, properties: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.request.properties = properties
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        self
    }
}
impl IntoFuture for CreateTableBuilder {
    type Output = Result<TableInfo>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.create_table(&request).await })
    }
}
