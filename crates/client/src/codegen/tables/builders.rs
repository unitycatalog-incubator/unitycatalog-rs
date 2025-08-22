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
    pub(crate) fn new(
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
    ///The array of ColumnInfo definitions of the table's columns.
    pub fn with_columns<I>(mut self, columns: I) -> Self
    where
        I: IntoIterator<Item = ColumnInfo>,
    {
        self.request.columns = columns.into_iter().collect();
        self
    }
    ///Storage root URL for external table.
    pub fn with_storage_location(mut self, storage_location: impl Into<Option<String>>) -> Self {
        self.request.storage_location = storage_location.into();
        self
    }
    ///User-provided free-form text description.
    pub fn with_comment(mut self, comment: impl Into<Option<String>>) -> Self {
        self.request.comment = comment.into();
        self
    }
    ///A map of key-value properties attached to the securable.
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
/// Builder for creating requests
pub struct GetTableBuilder {
    client: TableClient,
    request: GetTableRequest,
}
impl GetTableBuilder {
    /// Create a new builder instance
    pub(crate) fn new(client: TableClient, full_name: impl Into<String>) -> Self {
        let request = GetTableRequest {
            full_name: full_name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    ///Whether delta metadata should be included in the response.
    pub fn with_include_delta_metadata(
        mut self,
        include_delta_metadata: impl Into<Option<bool>>,
    ) -> Self {
        self.request.include_delta_metadata = include_delta_metadata.into();
        self
    }
    ///Whether to include tables in the response for which the principal can only access selective metadata for
    pub fn with_include_browse(mut self, include_browse: impl Into<Option<bool>>) -> Self {
        self.request.include_browse = include_browse.into();
        self
    }
    ///Whether to include a manifest containing capabilities the table has.
    pub fn with_include_manifest_capabilities(
        mut self,
        include_manifest_capabilities: impl Into<Option<bool>>,
    ) -> Self {
        self.request.include_manifest_capabilities = include_manifest_capabilities.into();
        self
    }
}
impl IntoFuture for GetTableBuilder {
    type Output = Result<TableInfo>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.get_table(&request).await })
    }
}
/// Builder for creating requests
pub struct GetTableExistsBuilder {
    client: TableClient,
    request: GetTableExistsRequest,
}
impl GetTableExistsBuilder {
    /// Create a new builder instance
    pub(crate) fn new(client: TableClient, full_name: impl Into<String>) -> Self {
        let request = GetTableExistsRequest {
            full_name: full_name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for GetTableExistsBuilder {
    type Output = Result<GetTableExistsResponse>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.get_table_exists(&request).await })
    }
}
