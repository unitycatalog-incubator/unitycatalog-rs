use std::collections::HashMap;

use futures::stream::BoxStream;
use futures::{StreamExt, TryStreamExt};

use super::tables::{TableClient, TableClientBase};
pub(super) use crate::api::schemas::SchemaClient as SchemaClientBase;
use crate::client::utils::hash_map_to_struct;
use crate::models::schemas::v1::*;
use crate::models::tables::v1::{
    ColumnInfo, CreateTableRequest, DataSourceFormat, TableInfo, TableType,
};
use crate::utils::stream_paginated;
use crate::{Error, Result};

impl SchemaClientBase {
    pub fn list(
        &self,
        catalog_name: impl Into<String>,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<SchemaInfo>> {
        let max_results = max_results.into();
        let catalog_name = catalog_name.into();
        stream_paginated(
            (catalog_name, max_results),
            move |(catalog_name, max_results), page_token| async move {
                let request = ListSchemasRequest {
                    catalog_name: catalog_name.clone(),
                    max_results,
                    page_token,
                    include_browse: None,
                };
                let res = self
                    .list_schemas(&request)
                    .await
                    .map_err(|e| Error::generic(e.to_string()))?;
                Ok((
                    res.schemas,
                    (catalog_name, max_results),
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

    pub(super) async fn create(&self, comment: Option<impl ToString>) -> Result<SchemaInfo> {
        let request = CreateSchemaRequest {
            catalog_name: self.catalog_name.clone(),
            name: self.schema_name.clone(),
            comment: comment.map(|s| s.to_string()),
            ..Default::default()
        };
        self.client.create_schema(&request).await
    }

    /// Create a new table in this schema.
    pub async fn create_table(
        &self,
        name: impl ToString,
        table_type: TableType,
        data_source_format: DataSourceFormat,
        columns: Vec<ColumnInfo>,
        storage_location: Option<impl ToString>,
        comment: Option<impl ToString>,
        properties: impl Into<Option<HashMap<String, String>>>,
    ) -> Result<TableInfo> {
        let request = CreateTableRequest {
            name: name.to_string(),
            schema_name: self.schema_name.clone(),
            catalog_name: self.catalog_name.clone(),
            table_type: table_type as i32,
            data_source_format: data_source_format as i32,
            columns,
            storage_location: storage_location.map(|s| s.to_string()),
            comment: comment.map(|c| c.to_string()),
            properties: properties.into().map(|m| hash_map_to_struct(m)),
        };
        let tables_client = super::tables::TableClientBase::new(
            self.client.client.clone(),
            self.client.base_url.clone(),
        );
        tables_client.create_table(&request).await
    }

    pub async fn get(&self) -> Result<SchemaInfo> {
        let request = GetSchemaRequest {
            full_name: format!("{}.{}", self.catalog_name, self.schema_name),
        };
        self.client.get_schema(&request).await
    }

    pub async fn update(
        &self,
        new_name: Option<impl ToString>,
        comment: Option<impl ToString>,
        properties: impl Into<Option<HashMap<String, String>>>,
    ) -> Result<SchemaInfo> {
        let request = UpdateSchemaRequest {
            full_name: format!("{}.{}", self.catalog_name, self.schema_name),
            new_name: new_name
                .map(|s| s.to_string())
                .unwrap_or_else(|| self.schema_name.clone()),
            comment: comment.map(|s| s.to_string()),
            properties: properties.into().map(|m| hash_map_to_struct(m)),
        };
        self.client.update_schema(&request).await
    }

    pub async fn delete(&self, force: impl Into<Option<bool>>) -> Result<()> {
        let request = DeleteSchemaRequest {
            full_name: format!("{}.{}", self.catalog_name, self.schema_name),
            force: force.into(),
        };
        tracing::debug!("Deleting schema {}", request.full_name);
        self.client.delete_schema(&request).await
    }
}
