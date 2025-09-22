use futures::stream::BoxStream;
use futures::{StreamExt, TryStreamExt};
use unitycatalog_common::models::tables::v1::*;

use super::utils::stream_paginated;
use crate::Result;
use crate::codegen::tables::DeleteTableBuilder;
pub(super) use crate::codegen::tables::TableClient as TableClientBase;
use crate::codegen::tables::builders::GetTableBuilder;

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
            move |(catalog_name, schema_name_pattern, table_name_pattern, mut max_results),
                  page_token| async move {
                let request = ListTableSummariesRequest {
                    catalog_name: catalog_name.clone(),
                    schema_name_pattern: schema_name_pattern.clone(),
                    table_name_pattern: table_name_pattern.clone(),
                    page_token,
                    max_results,
                    include_manifest_capabilities: None,
                };
                let res = self.list_table_summaries(&request).await?;

                // Update max_results for next page based on items received
                if let Some(ref mut remaining) = max_results {
                    *remaining -= res.tables.len() as i32;
                    if *remaining <= 0 {
                        max_results = Some(0);
                    }
                }

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
        include_browse: impl Into<Option<bool>>,
        include_manifest_capabilities: impl Into<Option<bool>>,
    ) -> BoxStream<'_, Result<Table>> {
        let max_results = max_results.into();
        let catalog_name = catalog_name.into();
        let schema_name = schema_name.into();
        let include_delta_metadata = include_delta_metadata.into();
        let omit_columns = omit_columns.into();
        let omit_properties = omit_properties.into();
        let omit_username = omit_username.into();
        let include_browse = include_browse.into();
        let include_manifest_capabilities = include_manifest_capabilities.into();
        stream_paginated(
            (catalog_name, schema_name, max_results),
            move |(catalog_name, schema_name, mut max_results), page_token| async move {
                let request = ListTablesRequest {
                    catalog_name: catalog_name.clone(),
                    schema_name: schema_name.clone(),
                    include_delta_metadata,
                    omit_columns,
                    omit_properties,
                    omit_username,
                    max_results,
                    page_token,
                    include_browse,
                    include_manifest_capabilities,
                };
                let res = self.list_tables(&request).await?;

                // Update max_results for next page based on items received
                if let Some(ref mut remaining) = max_results {
                    *remaining -= res.tables.len() as i32;
                    if *remaining <= 0 {
                        max_results = Some(0);
                    }
                }

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

    /// Get a table using the builder pattern.
    pub fn get(&self) -> GetTableBuilder {
        GetTableBuilder::new(self.client.clone(), &self.full_name)
    }

    pub fn delete(&self) -> DeleteTableBuilder {
        DeleteTableBuilder::new(self.client.clone(), self.full_name.clone())
    }
}
