#![allow(unused_mut)]
use super::client::*;
use crate::{error::Result, utils::stream_paginated};
use futures::{Stream, future::BoxFuture};
use std::future::IntoFuture;
use unitycatalog_common::models::tables::v1::*;
/// Builder for creating requests
pub struct ListTableSummariesBuilder {
    client: TableClient,
    request: ListTableSummariesRequest,
}
impl ListTableSummariesBuilder {
    /// Create a new builder instance
    pub(crate) fn new(client: TableClient, catalog_name: impl Into<String>) -> Self {
        let request = ListTableSummariesRequest {
            catalog_name: catalog_name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    ///A sql LIKE pattern (% and _) for schema names. All schemas will be returned if not set or empty.
    pub fn with_schema_name_pattern(
        mut self,
        schema_name_pattern: impl Into<Option<String>>,
    ) -> Self {
        self.request.schema_name_pattern = schema_name_pattern.into();
        self
    }
    ///A sql LIKE pattern (% and _) for table names. All tables will be returned if not set or empty.
    pub fn with_table_name_pattern(
        mut self,
        table_name_pattern: impl Into<Option<String>>,
    ) -> Self {
        self.request.table_name_pattern = table_name_pattern.into();
        self
    }
    ///The maximum number of results per page that should be returned.
    pub fn with_max_results(mut self, max_results: impl Into<Option<i32>>) -> Self {
        self.request.max_results = max_results.into();
        self
    }
    ///Opaque pagination token to go to next page based on previous query.
    pub fn with_page_token(mut self, page_token: impl Into<Option<String>>) -> Self {
        self.request.page_token = page_token.into();
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
impl IntoFuture for ListTableSummariesBuilder {
    type Output = Result<ListTableSummariesResponse>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.list_table_summaries(&request).await })
    }
}
/// Builder for creating requests
pub struct ListTablesBuilder {
    client: TableClient,
    request: ListTablesRequest,
}
impl ListTablesBuilder {
    /// Create a new builder instance
    pub(crate) fn new(
        client: TableClient,
        schema_name: impl Into<String>,
        catalog_name: impl Into<String>,
    ) -> Self {
        let request = ListTablesRequest {
            schema_name: schema_name.into(),
            catalog_name: catalog_name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    ///The maximum number of results per page that should be returned.
    pub fn with_max_results(mut self, max_results: impl Into<Option<i32>>) -> Self {
        self.request.max_results = max_results.into();
        self
    }
    ///Opaque pagination token to go to next page based on previous query.
    pub fn with_page_token(mut self, page_token: impl Into<Option<String>>) -> Self {
        self.request.page_token = page_token.into();
        self
    }
    ///Whether delta metadata should be included in the response.
    pub fn with_include_delta_metadata(
        mut self,
        include_delta_metadata: impl Into<Option<bool>>,
    ) -> Self {
        self.request.include_delta_metadata = include_delta_metadata.into();
        self
    }
    ///Whether to omit the columns of the table from the response or not.
    pub fn with_omit_columns(mut self, omit_columns: impl Into<Option<bool>>) -> Self {
        self.request.omit_columns = omit_columns.into();
        self
    }
    ///Whether to omit the properties of the table from the response or not.
    pub fn with_omit_properties(mut self, omit_properties: impl Into<Option<bool>>) -> Self {
        self.request.omit_properties = omit_properties.into();
        self
    }
    ///Whether to omit the username of the table (e.g. owner, updated_by, created_by) from the response or not.
    pub fn with_omit_username(mut self, omit_username: impl Into<Option<bool>>) -> Self {
        self.request.omit_username = omit_username.into();
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
    /// Convert paginated request into stream of results
    pub(crate) fn into_stream(&self) -> impl Stream<Item = Result<ListTablesResponse>> {
        let request = self.request.clone();
        stream_paginated(request, move |mut request, page_token| async move {
            request.page_token = page_token;
            let res = self.client.list_tables(&request).await?;
            if let Some(ref mut remaining) = request.max_results {
                *remaining -= res.tables.len() as i32;
                if *remaining <= 0 {
                    request.max_results = Some(0);
                }
            }
            let next_page_token = res.next_page_token.clone();
            Ok((res, request, next_page_token))
        })
    }
}
impl IntoFuture for ListTablesBuilder {
    type Output = Result<ListTablesResponse>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.list_tables(&request).await })
    }
}
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
/// Builder for creating requests
pub struct DeleteTableBuilder {
    client: TableClient,
    request: DeleteTableRequest,
}
impl DeleteTableBuilder {
    /// Create a new builder instance
    pub(crate) fn new(client: TableClient, full_name: impl Into<String>) -> Self {
        let request = DeleteTableRequest {
            full_name: full_name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for DeleteTableBuilder {
    type Output = Result<()>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.delete_table(&request).await })
    }
}
