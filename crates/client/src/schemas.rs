use futures::stream::BoxStream;
use futures::{StreamExt, TryStreamExt};
use unitycatalog_common::models::schemas::v1::*;
use unitycatalog_common::models::tables::v1::{DataSourceFormat, TableType};

use super::tables::{TableClient, TableClientBase};
use super::utils::stream_paginated;
use crate::Result;
use crate::codegen::schemas::DeleteSchemaBuilder;
pub(super) use crate::codegen::schemas::SchemaClient as SchemaClientBase;
use crate::codegen::schemas::builders::{
    CreateSchemaBuilder, GetSchemaBuilder, UpdateSchemaBuilder,
};
use crate::codegen::tables::builders::CreateTableBuilder;

impl SchemaClientBase {
    pub fn list(
        &self,
        catalog_name: impl Into<String>,
        max_results: impl Into<Option<i32>>,
        include_browse: impl Into<Option<bool>>,
    ) -> BoxStream<'_, Result<SchemaInfo>> {
        let max_results = max_results.into();
        let catalog_name = catalog_name.into();
        let include_browse = include_browse.into();
        stream_paginated(
            (catalog_name, max_results, include_browse),
            move |(catalog_name, mut max_results, include_browse), page_token| async move {
                let request = ListSchemasRequest {
                    catalog_name: catalog_name.clone(),
                    max_results,
                    page_token,
                    include_browse: None,
                };
                let res = self.list_schemas(&request).await?;

                // Update max_results for next page based on items received
                if let Some(ref mut remaining) = max_results {
                    *remaining -= res.schemas.len() as i32;
                    if *remaining <= 0 {
                        max_results = Some(0);
                    }
                }

                Ok((
                    res.schemas,
                    (catalog_name, max_results, include_browse),
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
pub struct SchemaClient {
    catalog_name: String,
    schema_name: String,
    client: SchemaClientBase,
}

impl SchemaClient {
    pub fn new(
        catalog_name: impl ToString,
        schema_name: impl ToString,
        client: SchemaClientBase,
    ) -> Self {
        Self {
            catalog_name: catalog_name.to_string(),
            schema_name: schema_name.to_string(),
            client,
        }
    }

    pub fn table(&self, name: impl Into<String>) -> TableClient {
        TableClient::new(
            format!("{}.{}.{}", self.catalog_name, self.schema_name, name.into()),
            TableClientBase::new(self.client.client.clone(), self.client.base_url.clone()),
        )
    }

    /// Create a new schema using the builder pattern.
    pub fn create(&self) -> CreateSchemaBuilder {
        CreateSchemaBuilder::new(self.client.clone(), &self.schema_name, &self.catalog_name)
    }

    /// Create a new table in this schema using the builder pattern.
    pub fn create_table(
        &self,
        name: impl ToString,
        table_type: TableType,
        data_source_format: DataSourceFormat,
    ) -> CreateTableBuilder {
        let tables_client = super::tables::TableClientBase::new(
            self.client.client.clone(),
            self.client.base_url.clone(),
        );
        CreateTableBuilder::new(
            tables_client,
            name.to_string(),
            &self.schema_name,
            &self.catalog_name,
            table_type,
            data_source_format,
        )
    }

    /// Get a schema using the builder pattern.
    pub fn get(&self) -> GetSchemaBuilder {
        GetSchemaBuilder::new(
            self.client.clone(),
            format!("{}.{}", self.catalog_name, self.schema_name),
        )
    }

    /// Update this schema using the builder pattern.
    pub fn update(&self) -> UpdateSchemaBuilder {
        UpdateSchemaBuilder::new(
            self.client.clone(),
            format!("{}.{}", self.catalog_name, self.schema_name),
        )
    }

    pub fn delete(&self) -> DeleteSchemaBuilder {
        DeleteSchemaBuilder::new(
            self.client.clone(),
            format!("{}.{}", self.catalog_name, self.schema_name),
        )
    }
}
