use futures::stream::BoxStream;
use futures::{StreamExt, TryStreamExt};
use unitycatalog_common::models::catalogs::v1::*;

use super::schemas::{SchemaClient, SchemaClientBase};
use super::utils::stream_paginated;
use crate::Result;
pub(super) use crate::codegen::catalogs::CatalogClient as CatalogClientBase;
use crate::codegen::catalogs::builders::{GetCatalogBuilder, UpdateCatalogBuilder};
use crate::codegen::schemas::CreateSchemaBuilder;

impl CatalogClientBase {
    pub fn list(&self, max_results: impl Into<Option<i32>>) -> BoxStream<'_, Result<CatalogInfo>> {
        let max_results = max_results.into();
        stream_paginated(max_results, move |mut max_results, page_token| async move {
            let request = ListCatalogsRequest {
                max_results,
                page_token,
            };
            let res = self.list_catalogs(&request).await?;

            // Update max_results for next page based on items received
            if let Some(ref mut remaining) = max_results {
                *remaining -= res.catalogs.len() as i32;
                if *remaining <= 0 {
                    max_results = Some(0);
                }
            }

            Ok((res.catalogs, max_results, res.next_page_token))
        })
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }
}

#[derive(Clone)]
pub struct CatalogClient {
    name: String,
    client: CatalogClientBase,
}

impl CatalogClient {
    pub fn new(name: impl ToString, client: CatalogClientBase) -> Self {
        Self {
            name: name.to_string(),
            client,
        }
    }

    /// Create a new schema in this catalog.
    pub fn create_schema(&self, name: impl ToString) -> CreateSchemaBuilder {
        let schemas_client = super::schemas::SchemaClientBase::new(
            self.client.client.clone(),
            self.client.base_url.clone(),
        );
        CreateSchemaBuilder::new(schemas_client, name.to_string(), &self.name)
    }

    /// Get a schema client for a schema contained in this catalog.
    pub fn schema(&self, name: impl ToString) -> SchemaClient {
        SchemaClient::new(
            &self.name,
            name,
            SchemaClientBase::new(self.client.client.clone(), self.client.base_url.clone()),
        )
    }

    /// Get a catalog using the builder pattern.
    pub fn get(&self) -> GetCatalogBuilder {
        GetCatalogBuilder::new(self.client.clone(), &self.name)
    }

    pub fn update(&self) -> UpdateCatalogBuilder {
        UpdateCatalogBuilder::new(self.client.clone(), &self.name)
    }

    pub async fn delete(&self, force: impl Into<Option<bool>>) -> Result<()> {
        let request = DeleteCatalogRequest {
            name: self.name.clone(),
            force: force.into(),
        };
        self.client.delete_catalog(&request).await
    }
}
