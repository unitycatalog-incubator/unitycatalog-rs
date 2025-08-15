use futures::stream::BoxStream;
use futures::{StreamExt, TryStreamExt};

pub(super) use crate::api::codegen::tables::TableClient as TableClientBase;
use crate::models::tables::v1::*;
use crate::utils::stream_paginated;
use crate::{Error, Result};

impl TableClientBase {
    pub fn list_summaries(
        &self,
        catalog_name: impl Into<String>,
        schema_name_pattern: Option<impl ToString>,
        table_name_pattern: Option<impl ToString>,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<TableSummary>> {
        let max_results = max_results.into();
        let catalog_name = catalog_name.into();
        let schema_name_pattern = schema_name_pattern.map(|s| s.to_string());
        let table_name_pattern = table_name_pattern.map(|s| s.to_string());
        stream_paginated(
            (
                catalog_name,
                schema_name_pattern,
                table_name_pattern,
                max_results,
            ),
            move |(catalog_name, schema_name_pattern, table_name_pattern, max_results),
                  page_token| async move {
                let request = ListTableSummariesRequest {
                    catalog_name: catalog_name.clone(),
                    schema_name_pattern: schema_name_pattern.clone(),
                    table_name_pattern: table_name_pattern.clone(),
                    page_token,
                    max_results: None,
                    include_manifest_capabilities: None,
                };
                let res = self
                    .list_table_summaries(&request)
                    .await
                    .map_err(|e| Error::generic(e.to_string()))?;
                Ok((
                    res.tables,
                    (
                        catalog_name,
                        schema_name_pattern,
                        table_name_pattern,
                        max_results,
                    ),
                    res.next_page_token,
                ))
            },
        )
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }

    pub fn list(
        &self,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
        max_results: impl Into<Option<i32>>,
        include_delta_metadata: impl Into<Option<bool>>,
        omit_columns: impl Into<Option<bool>>,
        omit_properties: impl Into<Option<bool>>,
        omit_username: impl Into<Option<bool>>,
    ) -> BoxStream<'_, Result<TableInfo>> {
        let max_results = max_results.into();
        let catalog_name = catalog_name.into();
        let schema_name = schema_name.into();
        let include_delta_metadata = include_delta_metadata.into();
        let omit_columns = omit_columns.into();
        let omit_properties = omit_properties.into();
        let omit_username = omit_username.into();
        stream_paginated(
            (catalog_name, schema_name, max_results),
            move |(catalog_name, schema_name, max_results), page_token| async move {
                let request = ListTablesRequest {
                    catalog_name: catalog_name.clone(),
                    schema_name: schema_name.clone(),
                    include_delta_metadata,
                    omit_columns,
                    omit_properties,
                    omit_username,
                    max_results,
                    page_token,
                    include_browse: None,
                    include_manifest_capabilities: None,
                };
                let res = self
                    .list_tables(&request)
                    .await
                    .map_err(|e| Error::generic(e.to_string()))?;
                Ok((
                    res.tables,
                    (catalog_name, schema_name, max_results),
                    res.next_page_token,
                ))
            },
        )
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }
}

#[derive(Clone)]
pub struct TableClient {
    full_name: String,
    client: TableClientBase,
}

impl TableClient {
    pub fn new(full_name: impl ToString, client: TableClientBase) -> Self {
        Self {
            full_name: full_name.to_string(),
            client,
        }
    }

    pub async fn get(
        &self,
        include_delta_metadata: impl Into<Option<bool>>,
        include_browse: impl Into<Option<bool>>,
        include_manifest_capabilities: impl Into<Option<bool>>,
    ) -> Result<TableInfo> {
        let request = GetTableRequest {
            full_name: self.full_name.clone(),
            include_delta_metadata: include_delta_metadata.into(),
            include_browse: include_browse.into(),
            include_manifest_capabilities: include_manifest_capabilities.into(),
        };
        self.client.get_table(&request).await
    }

    pub async fn delete(&self) -> Result<()> {
        let request = DeleteTableRequest {
            full_name: self.full_name.clone(),
        };
        self.client.delete_table(&request).await
    }
}
